#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::panic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
// road to clippy::pedantic
#![deny(clippy::pedantic)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

mod raster_tile_join;

use crate::raster_tile_join::join_raster_children;
use clap::Parser;
use futures::StreamExt;
use image::GenericImage;
use indoc::indoc;
use rusqlite::{Connection, Result as RusqliteResult};
use std::io::Cursor;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use utiles::lager::{init_tracing, LagerConfig, LagerLevel};
use utiles::mbt::{
    MbtStreamWriterSync, MbtType, MbtWriterStats, Mbtiles, MbtilesAsync,
    MbtilesClientAsync,
};
use utiles::sqlite::AsyncSqliteConn;
use utiles::Tile;
use utiles::UtilesResult;

struct ImgJoiner {
    pub tl: Option<image::DynamicImage>,
    pub tr: Option<image::DynamicImage>,
    pub bl: Option<image::DynamicImage>,
    pub br: Option<image::DynamicImage>,
}
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

    /// optimize level
    #[arg(required = false, long, short, default_value = "2")]
    pub(crate) opt: u8,

    // optimize alpha channel (default: false)
    // #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    // pub(crate) alpha: bool,

    // /// palette-reduction optimization (default: false)
    // #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    // pub(crate) palette: bool,
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
impl ImgJoiner {
    pub fn preflight(
        &self,
    ) -> anyhow::Result<
        //     dims
        (u32, u32),
    > {
        //     all images are the same size
        // all are not none
        if self.tl.is_none()
            && self.tr.is_none()
            && self.bl.is_none()
            && self.br.is_none()
        {
            return Err(anyhow::anyhow!("one or more images are missing"));
        }

        Ok((256, 256))
    }
    pub fn join(&self) -> anyhow::Result<image::DynamicImage> {
        let (w, h) = self.preflight()?;

        let out_w = w * 2;
        let out_h = h * 2;

        let mut img_buf_b = image::DynamicImage::new_rgba8(out_w, out_h);

        // if tl is not none, copy it to the top left
        if let Some(tl) = &self.tl {
            img_buf_b.copy_from(tl, 0, 0)?;
        }

        // if tr is not none, copy it to the top right
        if let Some(tr) = &self.tr {
            img_buf_b.copy_from(tr, w, 0)?;
        }
        // if bl is not none, copy it to the bottom left
        if let Some(bl) = &self.bl {
            img_buf_b.copy_from(bl, 0, h)?;
        }
        // if br is not none, copy it to the bottom right
        if let Some(br) = &self.br {
            img_buf_b.copy_from(br, h, w)?;
        }
        Ok(img_buf_b)
    }
}

/// Creates and returns an async `Receiver` of items derived from rows in the DB.
/// `T` is the custom output type.
/// `F` is a closure that maps a `rusqlite::Row` to `T`.
pub fn sqlite_query_tokio_receiver<T, F, C>(
    mbt: C,
    // &MbtilesClientAsync,
    query_override: &str,
    row_mapper: F,
) -> UtilesResult<tokio::sync::mpsc::Receiver<T>>
where
    // The row_mapper must be callable from inside a `spawn_blocking`.
    F: Fn(&rusqlite::Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
    C: AsyncSqliteConn + Clone + 'static,
{
    // Create a channel for streaming out `T` items.
    let (tx, rx) = tokio::sync::mpsc::channel::<T>(100);
    let query = query_override.to_string();
    // clone handle for spawned task.
    let mbt_clone = mbt.clone();

    tokio::spawn(async move {
        // Perform the DB connection + row iteration on the blocking thread (via `.conn()`).
        let result = mbt_clone
            .conn(move |conn: &Connection| -> RusqliteResult<()> {
                let mut stmt = conn.prepare(&query)?;
                // Map each DB row into T via `row_mapper`.
                let rows_iter = stmt.query_map([], |row| {
                    // Convert row -> T
                    let item = row_mapper(row)?;

                    // Send to the channel (blocking_send is fine in this context).
                    if let Err(e) = tx.blocking_send(item) {
                        warn!("channel send error: {:?}", e);
                    }

                    Ok(())
                })?;

                // consume all rows
                for row_result in rows_iter {
                    // You can optionally handle row-level errors here
                    if let Err(e) = row_result {
                        error!("row error: {:?}", e);
                    }
                }
                Ok(())
            })
            .await;

        if let Err(e) = result {
            error!("make_stream_rx: DB error: {:?}", e);
        }
    });
    Ok(rx)
}
pub fn sqlite_query_tokio_receiver_stream<T, F, C>(
    mbt: C,
    query_override: &str,
    row_mapper: F,
) -> UtilesResult<ReceiverStream<T>>
where
    // The row_mapper must be callable from inside a `spawn_blocking`.
    F: Fn(&rusqlite::Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
    C: AsyncSqliteConn + Clone + 'static,
{
    let tokio_rx = sqlite_query_tokio_receiver(mbt, query_override, row_mapper)?;
    Ok(ReceiverStream::new(tokio_rx))
}
// pub fn sqlite_query_tokio_receiver_stream<T, F>(
//     mbt: &MbtilesClientAsync,
//     query_override:&str,
//     row_mapper: F,
// ) -> UtilesResult<ReceiverStream<T>>
// where
//     F: Fn(&rusqlite::Row) -> RusqliteResult<T> + Send + Sync + 'static,
//     T: Send + 'static,
// {
//     let tokio_rx = sqlite_query_tokio_receiver(&mbt, query_override, row_mapper)?;
//     Ok(ReceiverStream::new(tokio_rx))
// }

const QUERY: &str = indoc! {r#"
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
"#
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

fn map_four_tile_row(row: &rusqlite::Row) -> rusqlite::Result<TileChildrenRow> {
    Ok(TileChildrenRow {
        parent_z: row.get("parent_z")?,
        parent_x: row.get("parent_x")?,
        parent_y: row.get("parent_y")?,
        child_0: row.get("child_0")?,
        child_1: row.get("child_1")?,
        child_2: row.get("child_2")?,
        child_3: row.get("child_3")?,
    })
}

fn load_image_from_memory(data: &[u8]) -> anyhow::Result<image::DynamicImage> {
    image::load_from_memory(data)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))
}

fn join_tmp(children: TileChildrenRow) -> anyhow::Result<Vec<u8>> {
    let raster_children_struct = raster_tile_join::RasterChildren {
        child_0: children.child_0.as_deref(),
        child_1: children.child_1.as_deref(),
        child_2: children.child_2.as_deref(),
        child_3: children.child_3.as_deref(),
    };
    join_raster_children(&raster_children_struct)
}

fn join_images2(children: TileChildrenRow) -> anyhow::Result<(Tile, Vec<u8>)> {
    // Helper function to load an image from memory with error handling
    // TIL about `Option::transpose()` which is doppppe
    let top_left = children
        .child_0
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let top_right = children
        .child_1
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let bottom_left = children
        .child_2
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let bottom_right = children
        .child_3
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;

    // Join the images
    let joiner = ImgJoiner {
        tl: top_left,
        tr: top_right,
        bl: bottom_left,
        br: bottom_right,
    };

    let img_buf = joiner.join()?;

    // Buffer the result in memory
    let mut bytes: Vec<u8> = Vec::new();
    // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

    Ok((
        Tile::new(children.parent_x, children.parent_y, children.parent_z),
        bytes,
    ))
}

pub async fn utiles_doubledown_main(args: &Cli) -> anyhow::Result<()> {
    info!("utiles-doubledown");
    debug!("args: {:?}", args);
    let mbt = MbtilesClientAsync::open_existing(&args.src).await?;
    mbt.assert_mbtiles().await?;
    // 2) Open or create the destination MBTiles
    // let dst = Mbtiles::from(dst_mbtiles);
    let dst_exists = std::fs::metadata(&args.dst).is_ok();
    if dst_exists {
        if args.force {
            std::fs::remove_file(&args.dst)?;
        } else {
            return Err(anyhow::anyhow!("dst exists, use --force to overwrite"));
        }
    }
    let dst = Mbtiles::open_new(&args.dst, Option::from(MbtType::Norm))?;

    let src_rows = mbt.metadata_rows().await?;
    for row in src_rows {
        info!("row: {:?}", row);
    }

    let mut src_rows = mbt.metadata_rows().await?;

    for row in &mut src_rows {
        if row.name == "minzoom" && row.value != "0" {
            let minzoom = row.value.parse::<u8>()?;
            let new_minzoom = minzoom - 1;
            row.value = new_minzoom.to_string();
        }
        if row.name == "maxzoom" {
            let maxzoom = row.value.parse::<u8>()?;
            // adjust the maxzoom because we're double downing so -1
            let new_maxzoom = maxzoom - 1;
            // info!("maxzoom: {:?}", maxzoom);
            row.value = new_maxzoom.to_string();
        }
        if row.name == "tilesize" {
            let tilesize = row.value.parse::<u32>()?;
            let new_tilesize = tilesize * 2;
            row.value = new_tilesize.to_string();
        }
        info!("row: {:?}", row);
    }

    dst.metadata_set_from_vec(&src_rows)?;
    // let thingystream =
    //     sqlite_query_tokio_receiver(mbt, QUERY, map_four_tile_row)?;
    // let stream = ReceiverStream::new(thingystream);
    let stream = sqlite_query_tokio_receiver_stream(mbt, QUERY, map_four_tile_row)?;
    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(1000);
    // mbt-writer stream....
    let mut writer = MbtStreamWriterSync {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst,
        stats: MbtWriterStats::default(),
    };

    let jobs = usize::from(args.jobs.unwrap_or(4));
    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        stream
            .for_each_concurrent(jobs, |d| {
                let tx_writer = tx_writer.clone();
                // let tx_progress = tx_progress.clone();
                // let initial_size = tile_data.len() as i64;

                async move {
                    let new_tile = Tile::new(d.parent_x, d.parent_y, d.parent_z);
                    let blocking_res =
                        tokio::task::spawn_blocking(move || join_tmp(d)).await;
                    match blocking_res {
                        Err(je) => {
                            warn!("join-error: {:?}", je);
                        }
                        Ok(imgjoin_result) => match imgjoin_result {
                            Ok(image_bytes) => {
                                let send_res =
                                    tx_writer.send((new_tile, image_bytes, None)).await;
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

    // wait for the writer and the processor to finish
    let (proc_res, write_res) = tokio::join!(proc_future, writer.write());
    if let Err(e) = proc_res {
        warn!("proc_res: {:?}", e);
    }
    if let Err(e) = write_res {
        warn!("write_res: {:?}", e);
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
    let res = utiles_doubledown_main(&args).await;
    res.map_err(|e| {
        error!("{}", e);
        e.into()
    })
}

#[tokio::main]
async fn main() {
    println!("utiles ~ dev");
    tokio_double_down().await.expect("utiles-doubledown failed");
}
