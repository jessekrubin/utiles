use clap::Parser;
use futures::StreamExt; // You may remove this if no longer needed
use image::{GenericImage, GenericImageView};
use indoc::indoc;
use crossbeam::channel::{bounded, Receiver};

use rayon::prelude::*;
use rusqlite::{Connection, Result as RusqliteResult, Row};
use std::io::Cursor;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use utiles::img::webpify_image; // or if not needed, remove
use utiles::lager::{init_tracing, LagerConfig, LagerLevel};
use utiles::mbt::{MbtStreamWriterSync, MbtType, MbtWriterStats, Mbtiles, MbtilesAsync};
use utiles::sqlite::AsyncSqliteConn; // remove if not needed
use utiles::UtilesResult;
use utiles_core::{utile_yup, Tile};

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

    /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
    #[arg(required = false, long, short)]
    pub jobs: Option<usize>,

    /// quiet
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) quiet: bool,

    /// force overwrite dst if exists
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) force: bool,
}

impl ImgJoiner {
    pub fn preflight(&self) -> anyhow::Result<(u32, u32)> {
        if self.tl.is_none() && self.tr.is_none() && self.bl.is_none() && self.br.is_none() {
            return Err(anyhow::anyhow!("one or more images are missing"));
        }
        Ok((256, 256))
    }

    pub fn join(&self) -> anyhow::Result<image::DynamicImage> {
        let (w, h) = self.preflight()?;
        let out_w = w * 2;
        let out_h = h * 2;

        let mut img_buf_b = image::DynamicImage::new_rgba8(out_w, out_h);

        if let Some(tl) = &self.tl {
            img_buf_b.copy_from(tl, 0, 0)?;
        }
        if let Some(tr) = &self.tr {
            img_buf_b.copy_from(tr, w, 0)?;
        }
        if let Some(bl) = &self.bl {
            img_buf_b.copy_from(bl, 0, h)?;
        }
        if let Some(br) = &self.br {
            // Notice the mismatch in your original snippet:
            // the old code had `img_buf_b.copy_from(br, h, w)?;` which likely was an error
            // (the arguments are `x, y`). Correct usage is `copy_from(&br, x= w, y= h)`.
            // If that was unintentional, fix it:
            img_buf_b.copy_from(br, w, h)?;
        }
        Ok(img_buf_b)
    }
}

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
"#};

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

/// Load a single tile’s bytes into a `DynamicImage`.
fn load_image_from_memory(data: &[u8]) -> anyhow::Result<image::DynamicImage> {
    image::load_from_memory(data)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))
}

/// Join the four child tiles into a new `(Tile, Vec<u8>)`.
fn join_images(children: TileChildrenRow) -> anyhow::Result<(Tile, Vec<u8>)> {
    let top_left = children.child_0.as_ref().map(|d| load_image_from_memory(d)).transpose()?;
    let top_right = children.child_1.as_ref().map(|d| load_image_from_memory(d)).transpose()?;
    let bottom_left = children.child_2.as_ref().map(|d| load_image_from_memory(d)).transpose()?;
    let bottom_right = children.child_3.as_ref().map(|d| load_image_from_memory(d)).transpose()?;

    let joiner = ImgJoiner {
        tl: top_left,
        tr: top_right,
        bl: bottom_left,
        br: bottom_right,
    };
    let img_buf = joiner.join()?;

    let mut bytes: Vec<u8> = Vec::new();
    // Use WebP or whatever format you need:
    img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    Ok((Tile::new(children.parent_x, children.parent_y, children.parent_z), bytes))
}

/// Spawns a thread that queries the rows and sends them down a Crossbeam channel.
///
/// - Returns a `Receiver<TileChildrenRow>` for consuming them.
/// - You can then `.into_iter().par_bridge()` that receiver to process in parallel.
fn gather_children_rows_stream(src: Mbtiles) -> anyhow::Result<Receiver<TileChildrenRow>> {
    // You can choose bounded or unbounded; choose an appropriate buffer size.
    let (tx, rx) = bounded::<TileChildrenRow>(1024);

    // Move or clone whatever you need into the thread
    std::thread::spawn(move || {
        // Acquire the DB connection
        let conn = match src.conn() {
            Ok(c) => c,
            Err(err) => {
                error!("DB conn error: {:?}", err);
                return ;
            }
        };

        // Prepare the statement
        let mut stmt = match conn.prepare(QUERY) {
            Ok(s) => s,
            Err(err) => {
                error!("prepare error: {:?}", err);
                return;
            }
        };

        // Query rows
        let rows_iter = match stmt.query_map([], map_four_tile_row) {
            Ok(iter) => iter,
            Err(err) => {
                error!("query_map error: {:?}", err);
                return;
            }
        };

        // Send each row into the channel
        for row_res in rows_iter {
            match row_res {
                Ok(item) => {
                    if tx.send(item).is_err() {
                        // The receiver was dropped; stop
                        break;
                    }
                }
                Err(e) => {
                    error!("row error: {:?}", e);
                }
            }
        }
        // When this scope finishes, `tx` is dropped => channel is closed.
    });

    Ok(rx)
}

/// Gather all the rows we need from the DB, synchronously.
fn gather_children_rows(src: &Mbtiles) -> anyhow::Result<Vec<TileChildrenRow>> {
    let conn = src.conn();
    let mut stmt = conn.prepare(QUERY)?;
    let rows_iter = stmt.query_map([], |row| map_four_tile_row(row))?;

    let mut rows = Vec::new();
    for row in rows_iter {
        match row {
            Ok(item) => rows.push(item),
            Err(e) => {
                error!("row error: {:?}", e);
            }
        }
    }
    Ok(rows)
}

/// Core “double-down” logic (no async, uses Rayon for parallelism).
fn utiles_512ify(args: &Cli) -> anyhow::Result<()> {
    info!("utiles ~ 512ify");
    debug!("args: {:?}", args);

    // 1) Open existing MBTiles (synchronously)
    let src = Mbtiles::open_existing(&args.src)?;

    // 2) Prepare (or create) the destination
    let dst_exists = std::fs::metadata(&args.dst).is_ok();
    if dst_exists {
        if args.force {
            std::fs::remove_file(&args.dst)?;
        } else {
            return Err(anyhow::anyhow!(
                "Destination exists, use --force to overwrite: {}",
                args.dst
            ));
        }
    }
    let mut dst = Mbtiles::open_new(&args.dst, Some(MbtType::Flat))?;

    // 3) Adjust metadata
    let mut src_rows = src.metadata_rows()?;
    for mut row in &mut src_rows {
        match row.name.as_str() {
            "minzoom" => {
                let minzoom = row.value.parse::<u8>()?;
                if minzoom > 0 {
                    let new_minzoom = minzoom - 1;
                    row.value = new_minzoom.to_string();
                }
            }
            "maxzoom" => {
                let maxzoom = row.value.parse::<u8>()?;
                let new_maxzoom = maxzoom.saturating_sub(1);
                row.value = new_maxzoom.to_string();
            }
            "tilesize" => {
                let tilesize = row.value.parse::<u32>()?;
                let new_tilesize = tilesize * 2;
                row.value = new_tilesize.to_string();
            }
            _ => {}
        }
        info!("metadata: {:?}", row);
    }
    dst.metadata_set_from_vec(&src_rows)?;

    // 4) Gather the “children” rows to be joined
    // let rows = gather_children_rows(&src)?;
    let row_rx = gather_children_rows_stream(src)?;

    // Ok(())
    // 5) Use rayon to process each row in parallel
    let jobs = args.jobs.unwrap_or_else(|| {
        // As a fallback, pick something like # of CPUs or 4
        std::cmp::max(4, num_cpus::get())
    });
    // Configure the Rayon global thread pool if you want:
    // rayon::ThreadPoolBuilder::new().num_threads(jobs).build_global()?;

    // let joined_tiles: Vec<(Tile, Vec<u8>)> = rows
    //     .into_par_iter()
    //     .map(|row| {
    //         match join_images(row) {
    //             Ok(result) => Some(result),
    //             Err(e) => {
    //                 warn!("join_images failed: {:?}", e);
    //                 None
    //             }
    //         }
    //     })
    //     .filter_map(|x| x)
    //     .collect();

    // info!("Processed {} tiles in parallel", joined_tiles.len());

    // 6) Write them all out with MbtStreamWriterSync
    // let mut writer = MbtStreamWriterSync {
    //     We won't use a Receiver now; just do direct writes in a loop
        // stream: Default::default(),
        // mbt: dst,
        // stats: MbtWriterStats::default(),
    // };

    // Synchronously insert each tile
    // for (tile, data) in joined_tiles {
        // The third field is an optional `tile_compression` if your `MbtStreamWriterSync`
        // signature is `(Tile, Vec<u8>, Option<String>)`. Adjust if needed.
        // dst.insert_tile_flat( &tile, &data)?;
    // }

    // info!("Done writing tiles. Stats = {:?}", writer.stats);
    Ok(())
}

pub fn main_rayon() -> anyhow::Result<()> {
    // Simple main instead of tokio::main
    let args = Cli::parse();

    let level = if args.debug {
        LagerLevel::Debug
    } else {
        LagerLevel::Info
    };

    // Initialize logging/tracing
    let logcfg = LagerConfig {
        json: false,
        level,
    };
    init_tracing(logcfg)?;

    println!("utiles ~ dev (rayon concurrency)");
    if let Err(e) = utiles_512ify(&args) {
        error!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}
