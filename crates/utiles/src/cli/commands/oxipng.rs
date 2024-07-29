use crate::cli::args::OxipngArgs;
use crate::img::oxipngify;
use crate::mbt::{make_tiles_stream, MbtStreamWriter, MbtWriterStats};
use crate::mbt::{Mbtiles, MbtilesAsync, MbtilesClientAsync};
use crate::UtilesResult;
use futures::StreamExt;
use indicatif;
use indicatif::ProgressStyle;
use std::time::Duration;
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};

pub async fn oxipng_main(args: OxipngArgs) -> UtilesResult<()> {
    let mbt = MbtilesClientAsync::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let total_count = mbt.tiles_count().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    let tiles_stream = make_tiles_stream(&mbt, None)?;

    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriter {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: MbtWriterStats::default(),
    };
    let jobs: usize = args.jobs.unwrap_or(4) as usize;

    let (tx_progress, mut rx_progress) = tokio::sync::mpsc::channel(100);

    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        tiles_stream
            .for_each_concurrent(jobs, |(tile, tile_data, _)| {
                let tx_writer = tx_writer.clone();
                let tx_progress = tx_progress.clone();
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
                                let size_diff =
                                    (initial_size as i64) - (final_size as i64);
                                debug!("size_diff: {}", size_diff);
                                let send_res = tx_writer.send((tile, img_result)).await;

                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                } else if let Err(e) = tx_progress.send(size_diff).await
                                {
                                    warn!("send error tx_progress: {:?}", e);
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
        pb.set_message("oxipng-ing");
        let pb_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        );
        if args.quiet {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }
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
            pb.set_message(format!("size-diff: {size_saved}"));
        }
        let total_size_str = size::Size::from_bytes(total_size_diff as u64).to_string();
        pb.finish_with_message(format!(
            "Processed {processed} tiles, saved {total_size_str} ({total_size_diff}b)"
        ));
    });

    let (cruncher_res, writer_res, progress_res) =
        join!(proc_future, writer.write(), progress_future);
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    cruncher_res?;
    writer_res?;
    progress_res?;
    Ok(())
}
