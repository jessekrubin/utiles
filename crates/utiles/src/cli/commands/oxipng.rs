use futures::StreamExt;
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};

use crate::cli::args::OxipngArgs;
use crate::img::oxipngify;
use crate::mbt::{make_tiles_stream, MbtStreamWriter, MbtWriterStats};
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

pub async fn oxipng_main(args: OxipngArgs) -> UtilesResult<()> {
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    let tiles_stream = make_tiles_stream(&mbt)?;

    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriter {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: MbtWriterStats::default(),
    };

    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        tiles_stream
            .for_each_concurrent(4, |(tile, tile_data)| {
                let tx_writer = tx_writer.clone();
                let oxipng_options = oxipng::Options::from_preset(args.opt);
                async move {
                    let initial_size = tile_data.len();
                    let blocking_res = tokio::task::spawn_blocking(move || {
                        oxipngify(&tile_data, &oxipng_options)
                    })
                    .await;
                    match blocking_res {
                        Err(je) => {
                            warn!("join-error: {:?}", je);
                        }
                        Ok(oxipngify_res) => match oxipngify_res {
                            Ok(img_result) => {
                                let final_size = img_result.len();
                                let size_diff: i32 =
                                    initial_size as i32 - final_size as i32;
                                debug!("size_diff: {}", size_diff);
                                let send_res = tx_writer.send((tile, img_result)).await;

                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                }
                            }
                            Err(e) => {
                                warn!("webpify_image: {:?}", e);
                            }
                        },
                    }
                }
            })
            .await;
    });

    let (result, writer_result) = join!(proc_future, writer.write_batched());
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    result?;
    writer_result?;
    Ok(())
}
