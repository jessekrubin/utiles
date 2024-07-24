use futures::StreamExt;
use indicatif::ProgressStyle;
use std::time::Duration;
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
    let total_count = mbt.tiles_count().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    dst_mbtiles.metadata_set("format", "webp")?;
    let tiles_stream = make_tiles_stream(&mbt)?;
    let (tx_progress, mut rx_progress) = tokio::sync::mpsc::channel(100);
    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriter {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: MbtWriterStats::default(),
    };
    let jobs: usize = args.jobs.unwrap_or(4) as usize;
    info!("webpify ~ total_count: {total_count} ~ jobs: {jobs}");
    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        tiles_stream
            .for_each_concurrent(jobs, |(tile, tile_data)| {
                let tx_writer = tx_writer.clone();
                let tx_progress = tx_progress.clone();
                let initial_size = tile_data.len();

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
                                let size_diff = initial_size - webp_bytes.len();
                                let send_res = tx_writer.send((tile, webp_bytes)).await;
                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                }
                                let send_res = tx_progress.send(size_diff).await;
                                if let Err(e) = send_res {
                                    warn!("progress send_res: {:?}", e);
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

    let progress_future = tokio::spawn(async move {
        let mut total_size_diff = 0;
        let mut processed = 0;
        let pb = indicatif::ProgressBar::new(total_count as u64);
        pb.set_message("webpify");
        let pb_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        );
        match pb_style {
            Err(e) => {
                warn!("pb_style error: {:?}", e);
            }
            Ok(s) => {
                pb.set_style(s);
            }
        }
        pb.enable_steady_tick(Duration::from_millis(100));
        while let Some(size_diff) = rx_progress.recv().await {
            total_size_diff += size_diff;
            processed += 1;
            pb.inc(1);

            let size_saved = size::Size::from_bytes(total_size_diff as u64).to_string();
            pb.set_message(format!("webpify ~ size-diff: {size_saved}"));
        }
        let total_size_str = size::Size::from_bytes(total_size_diff as u64).to_string();
        pb.finish_with_message(format!(
            "Processed {processed} tiles, saved {total_size_str} ({total_size_diff}b)"
        ));
    });
    let (result, writer_result, progress_res) =
        join!(proc_future, writer.write(), progress_future);
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    result?;
    writer_result?;
    progress_res?;

    Ok(())
}
