use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use utiles::tile_type::TileFormat;
use utiles::{
    lager::{init_tracing, LagerConfig, LagerLevel},
    mbt::{
        MbtStreamWriterSync, MbtWriterStats, Mbtiles, MbtilesAsync, MbtilesClientAsync,
    },
    tile_type::tiletype,
    UtilesResult,
};

#[derive(Debug, Parser)]
#[command(name = "utiles-oxipng")]
#[command(version = utiles::VERSION)]
#[command(max_term_width = 120)]
#[command(author)]
#[command(about = "oxipng-ify png-format mbtiles", long_about = None)]
struct Cli {
    /// debug
    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    /// mbtiles-like fspath
    #[arg(required = true)]
    pub src: String,

    /// destination mbtiles fspath
    #[arg(required = true)]
    pub dst: String,

    /// optimize level
    #[arg(required = false, long, short, default_value = "2")]
    pub(crate) opt: u8,

    /// optimize alpha channel (default: false)
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) alpha: bool,

    /// palette-reduction optimization (default: false)
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) palette: bool,

    /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
    #[arg(required = false, long, short)]
    pub jobs: Option<u8>,

    /// quiet
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) quiet: bool,
}

pub fn oxipngify(data: &[u8], options: &oxipng::Options) -> Result<Vec<u8>> {
    if let TileFormat::Png = tiletype(data).format {
        oxipng::optimize_from_memory(data, options).map_err(|e| e.into())
    } else {
        warn!("Unsupported image type");
        Ok(data.to_vec())
    }
}

async fn oxipng_main(args: Cli) -> UtilesResult<()> {
    let mbt = MbtilesClientAsync::open_existing(args.src.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let total_count = mbt.tiles_count().await?;
    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    let tiles_stream = mbt.tiles_stream(None)?;

    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriterSync {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: MbtWriterStats::default(),
    };
    let jobs: usize = args.jobs.unwrap_or(4) as usize;

    let (tx_progress, mut rx_progress) = tokio::sync::mpsc::channel(100);

    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        tiles_stream
            .for_each_concurrent(jobs, |(tile, tile_data)| {
                let tx_writer = tx_writer.clone();
                let tx_progress = tx_progress.clone();
                let mut oxipng_options = oxipng::Options::from_preset(args.opt);
                if args.alpha {
                    oxipng_options.optimize_alpha = true;
                }
                if args.palette {
                    oxipng_options.palette_reduction = true;
                }

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
                                let send_res =
                                    tx_writer.send((tile, img_result, None)).await;

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
        let pb_style = indicatif::ProgressStyle::with_template(
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
            let size_saved_msg = if total_size_diff > 0 {
                format!(
                    "-{}",
                    size::Size::from_bytes(total_size_diff.unsigned_abs())
                )
            } else {
                format!(
                    "+{}",
                    size::Size::from_bytes(total_size_diff.unsigned_abs())
                )
            };
            pb.set_message(format!("size-diff: {size_saved_msg}"));
        }
        let total_size_saved_str = if total_size_diff > 0 {
            format!(
                "-{}",
                size::Size::from_bytes(total_size_diff.unsigned_abs())
            )
        } else {
            format!(
                "+{}",
                size::Size::from_bytes(total_size_diff.unsigned_abs())
            )
        };
        pb.finish_with_message(format!(
            "Processed {processed} tiles, saved {total_size_saved_str} ({total_size_diff}b)"
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

#[tokio::main]
async fn main() -> Result<()> {
    debug!("utiles-oxipng");
    let args = Cli::parse();
    debug!("args: {:?}", args);

    let level = if args.debug {
        LagerLevel::Debug
    } else {
        LagerLevel::Info
    };
    let logcfg = LagerConfig { json: false, level };
    init_tracing(logcfg)?;
    let res = oxipng_main(args).await;
    res.map_err(|e| {
        error!("{}", e);
        e.into()
    })
}
