use futures::StreamExt;
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use crate::cli::args::WebpifyArgs;
use crate::img::webpify_image;
use crate::mbt::{make_tiles_stream, MbtStreamWriter, MbtWriterStats};
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

pub async fn webpify_main(args: WebpifyArgs) -> UtilesResult<()> {
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    dst_mbtiles.metadata_set("format", "webp")?;
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
                async move {
                    let blocking_res =
                        tokio::task::spawn_blocking(move || webpify_image(&tile_data))
                            .await;
                    match blocking_res {
                        Err(je) => {
                            warn!("join-error: {:?}", je);
                        }
                        Ok(webpify_result) => match webpify_result {
                            Ok(webp_bytes) => {
                                let send_res = tx_writer.send((tile, webp_bytes)).await;
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

    let (result, writer_result) = join!(proc_future, writer.write());
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    result?;
    writer_result?;
    Ok(())
}
