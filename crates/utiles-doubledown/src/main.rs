#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::panic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::must_use_candidate)]
// road to clippy::pedantic
#![deny(clippy::pedantic)]
#![expect(clippy::cast_possible_wrap)]

use crate::ProgressEvent::{Msg, SizeDiff};
use clap::Parser;
use futures::StreamExt;
use indoc::indoc;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use utiles::Tile;
use utiles::img::raster_tile_join;
use utiles::img::raster_tile_join::dynamic_img_2_webp;
use utiles::internal::cli_tools::open_new_overwrite;
use utiles::lager::{LagerConfig, LagerLevel, init_tracing};
use utiles::mbt::{
    MbtStreamWriterSync, MbtWriterStats, MbtilesAsync, MbtilesClientAsync,
};
use utiles::sqlite::InsertStrategy;

#[derive(Debug, Parser)]
#[command(name = "utiles-doubledown")]
#[command(version = utiles::VERSION)]
#[command(max_term_width = 120)]
#[command(author)]
#[command(about = "join-raster-tiles doubling each tile in a tiles-db", long_about = None)]
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

    /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
    #[arg(required = false, long, short)]
    pub jobs: Option<u8>,

    /// quiet
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) quiet: bool,

    /// force overwrite dst if exists
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) force: bool,
}

enum ProgressEvent {
    SizeDiff(i64),
    #[expect(dead_code)]
    Msg(String),
}

const QUERY: &str = indoc! {r"
WITH parent AS (SELECT DISTINCT (zoom_level - 1)  AS p_z,
                                (tile_column / 2) AS p_x,
                                (tile_row / 2)    AS p_y
                FROM tiles
                WHERE zoom_level > 0)
SELECT parent.p_z        AS parent_z,
       parent.p_x        AS parent_x,
       parent.p_y        AS parent_y,
       child_0.tile_data AS child_0, -- top/upper left ~ qk=0
       child_1.tile_data AS child_1, -- top/upper right ~ qk=1
       child_2.tile_data AS child_2, -- bottom/lower left ~ qk=2
       child_3.tile_data AS child_3  -- bottom/lower right ~ qk=3
FROM parent
         LEFT JOIN tiles child_0 ON child_0.zoom_level = parent.p_z + 1
    AND child_0.tile_column = parent.p_x * 2
    AND child_0.tile_row = parent.p_y * 2 + 1
         LEFT JOIN tiles child_1 ON child_1.zoom_level = parent.p_z + 1
    AND child_1.tile_column = parent.p_x * 2 + 1
    AND child_1.tile_row = parent.p_y * 2 + 1
         LEFT JOIN tiles child_2 ON child_2.zoom_level = parent.p_z + 1
    AND child_2.tile_column = parent.p_x * 2
    AND child_2.tile_row = parent.p_y * 2
         LEFT JOIN tiles child_3 ON child_3.zoom_level = parent.p_z + 1
    AND child_3.tile_column = parent.p_x * 2 + 1
    AND child_3.tile_row = parent.p_y * 2
"
};

#[derive(Debug)]
struct TileChildrenRow {
    parent_z: u8,
    parent_x: u32,
    parent_y: u32,
    child_0: Option<Vec<u8>>,
    child_1: Option<Vec<u8>>,
    child_2: Option<Vec<u8>>,
    child_3: Option<Vec<u8>>,
}

impl TileChildrenRow {
    fn data_vec(&self) -> Vec<&Option<Vec<u8>>> {
        vec![&self.child_0, &self.child_1, &self.child_2, &self.child_3]
    }
    fn total_size(&self) -> usize {
        self.data_vec().iter().fold(0, |acc, x| {
            acc + match x {
                Some(v) => v.len(),
                None => 0,
            }
        })
    }
}

fn map_four_tile_row(row: &rusqlite::Row) -> rusqlite::Result<TileChildrenRow> {
    let parent_z: u8 = row.get("parent_z")?;
    let parent_yup: u32 = row.get("parent_y")?;
    // that is upside-down ^ so must flip...
    let parent_y = (1 << parent_z) - 1 - parent_yup;
    Ok(TileChildrenRow {
        parent_z,
        parent_x: row.get("parent_x")?,
        parent_y,
        child_0: row.get("child_0")?,
        child_1: row.get("child_1")?,
        child_2: row.get("child_2")?,
        child_3: row.get("child_3")?,
    })
}

fn raster_join_tile_children_row(
    children: &TileChildrenRow,
) -> anyhow::Result<Vec<u8>> {
    let raster_children_struct = raster_tile_join::RasterChildren {
        child_0: children.child_0.as_deref(),
        child_1: children.child_1.as_deref(),
        child_2: children.child_2.as_deref(),
        child_3: children.child_3.as_deref(),
    };
    let start = std::time::Instant::now();
    let b = raster_tile_join::join_raster_children(&raster_children_struct)?;
    let elapsed = start.elapsed();
    debug!("join_raster_children elapsed: {:?}", elapsed);
    Ok(dynamic_img_2_webp(&b)?)
}

fn make_progress_future(
    mut rx_progress: tokio::sync::mpsc::Receiver<ProgressEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut total_size_diff: i64 = 0;
        let mut processed = 0;
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message("double-down");
        pb.enable_steady_tick(Duration::from_millis(100));
        while let Some(size_diff) = rx_progress.recv().await {
            match size_diff {
                SizeDiff(size_diff) => {
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
                    pb.set_message(format!(
                        "double-down ~ processed: {processed} ~ total-size-diff: {size_saved_msg}"
                    ));
                }
                Msg(msg) => {
                    pb.println(msg);
                }
            }
        }
        ///////////////////////////////////////////////////////////////////////
        // AND LAST BUT NOT LEAST, THE FINAL MESSAGE
        ///////////////////////////////////////////////////////////////////////
        let total_size_str = if total_size_diff > 0 {
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
            "Processed {processed} tiles, saved {total_size_str} ({total_size_diff}b)"
        ));
    })
}

async fn utiles_doubledown_main(args: Cli) -> anyhow::Result<()> {
    info!("utiles-doubledown");
    debug!("args: {:?}", args);
    let mbt = MbtilesClientAsync::open_existing(&args.src).await?;
    mbt.assert_mbtiles().await?;
    let dst = open_new_overwrite(&args.dst, args.force)?;
    let mut src_rows = mbt.metadata_rows().await?;
    for row in &mut src_rows {
        if row.name == "minzoom" && row.value != "0" {
            let new_minzoom = row.value.parse::<u8>().map(|mz| mz - 1)?;
            row.value = new_minzoom.to_string();
        }
        if row.name == "maxzoom" {
            let new_maxzoom = row.value.parse::<u8>().map(|mz| mz - 1)?;
            row.value = new_maxzoom.to_string();
        }
        if row.name == "tilesize" {
            let new_tilesize = row.value.parse::<u32>().map(|ts| ts * 2)?;
            row.value = new_tilesize.to_string();
        }
    }

    dst.metadata_set_from_vec(&src_rows)?;
    let stream = utiles::sqlite::streams::sqlite_query_tokio_receiver_stream(
        &mbt,
        QUERY,
        map_four_tile_row,
    )?;
    let (tx_progress, rx_progress) = tokio::sync::mpsc::channel::<ProgressEvent>(100);
    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    // mbt-writer stream....
    let mut writer = MbtStreamWriterSync {
        stream: ReceiverStream::new(rx_writer),
        on_conflict: InsertStrategy::None,
        mbt: dst,
        stats: MbtWriterStats::default(),
    };
    let progress_future = make_progress_future(rx_progress);
    let jobs = usize::from(args.jobs.unwrap_or(4));
    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        stream
            .for_each_concurrent(jobs, |d| {
                let tx_writer = tx_writer.clone();
                let tx_progress = tx_progress.clone();
                let initial_size = d.total_size() as i64;

                async move {
                    let new_tile = Tile::new(d.parent_x, d.parent_y, d.parent_z);
                    let blocking_res = tokio::task::spawn_blocking(move || {
                        raster_join_tile_children_row(&d)
                    })
                    .await;
                    match blocking_res {
                        Err(je) => {
                            warn!("join-error: {:?}", je);
                        }
                        Ok(imgjoin_result) => match imgjoin_result {
                            Ok(image_bytes) => {
                                let size_diff =
                                    initial_size - (image_bytes.len() as i64);
                                let send_res = tx_writer
                                    .send((new_tile, image_bytes, None).into())
                                    .await;
                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                }
                                let send_res =
                                    tx_progress.send(SizeDiff(size_diff)).await;
                                if let Err(e) = send_res {
                                    warn!("progress send_res: {:?}", e);
                                }
                            }
                            Err(e) => {
                                warn!("raster-join-error: {:?}", e);
                            }
                        },
                    }
                }
            })
            .await;
    });

    // wait for the writer and the processor to finish
    let (proc_res, write_res, progress_res) =
        tokio::join!(proc_future, writer.write(), progress_future);
    if let Err(e) = proc_res {
        warn!("proc_res: {:?}", e);
    }
    if let Err(e) = write_res {
        warn!("write_res: {:?}", e);
    }
    if let Err(e) = progress_res {
        warn!("progress_res: {:?}", e);
    }
    Ok(())
}

async fn tokio_double_down() -> anyhow::Result<()> {
    let args = Cli::parse();
    debug!("utiles-doubledown ~ args: {:?}", args);
    let level = if args.debug {
        LagerLevel::Debug
    } else {
        LagerLevel::Info
    };
    let logcfg = LagerConfig { json: false, level };
    init_tracing(logcfg)?;
    let res = utiles_doubledown_main(args).await;
    res.map_err(|e| {
        error!("{}", e);
        e
    })
}

#[tokio::main]
async fn main() {
    tokio_double_down().await.expect("utiles-doubledown failed");
}
