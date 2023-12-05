use std::cell::Cell;
use std::path::{Path, PathBuf};

// use tokio_stream::{self as stream, Stream};
use futures::stream::{self, StreamExt};
use tokio::fs;
use tracing::{debug, info, warn};
use utiles::mbtiles::MbtTileRow;
use utiles::{flipy, Tile, TileLike};
use serde_json;
use crate::args::CopyArgs;
use utilesqlite::Mbtiles;

// #[derive(Debug)]
// pub struct MbtTileRow {
//     zoom_level: u8,
//     tile_column: u32,
//     tile_row: u32,
//     tile_data: Vec<u8>,
// }
//
#[derive(Debug)]
pub struct WriterStats {
    pub nwritten: Cell<u32>,
}

#[derive(Debug)]
struct TilesFsWriter {
    root_dirpath: String,
    stats: WriterStats,
}

impl TilesFsWriter {
    pub fn new(root_dirpath: String) -> Self {
        Self {
            root_dirpath,
            stats: WriterStats {
                nwritten: Cell::new(0),
            },
        }
    }

    fn dirpath(&self, z: u8, x: u32) -> PathBuf {
        Path::new(&self.root_dirpath)
            .join(format!("{z}"))
            .join(format!("{x}"))
    }

    pub async fn mkdirpath(&self, z: u8, x: u32) {
        let dp = self.dirpath(z, x);
        let dp = dp.to_str().unwrap();
        fs::create_dir_all(dp).await.unwrap();
    }

    pub async fn write_tile(&self, tile: MbtTileRow) {
        let filepath = self.dirpath(tile.z(), tile.x()).join(format!(
            "{}.{}",
            flipy(tile.y(), tile.z()),
            tile.extension()
        ));
        debug!("filepath: {:?}", filepath);
        fs::write(filepath, tile.tile_data).await.unwrap();
        self.inc_nwritten();
    }

    pub fn inc_nwritten(&self) {
        let n = self.stats.nwritten.get();
        self.stats.nwritten.set(n + 1);
    }

    pub fn nwritten(&self) -> u32 {
        self.stats.nwritten.get()
    }
}

pub enum Source {
    Mbtiles(String),
}

pub enum Destination {
    // Mbtiles(String),
    Fs(String),
}

async fn copy_mbtiles2fs(mbtiles: String, output_dir: String) {
    let mbt = Mbtiles::from(mbtiles.as_ref());
    let start_time = std::time::Instant::now();
    let total_tiles: u32 = mbt
        .conn()
        .query_row("SELECT count(*) FROM tiles", [], |row| row.get(0))
        .unwrap();
    info!("finna write {total_tiles:?} from {mbtiles:?} to {output_dir:?}");
    let c = mbt.conn();

    let metadata_vec = mbt.metadata().unwrap();
    let metadata_str = serde_json::to_string_pretty(&metadata_vec).unwrap();
    println!("{}", metadata_str);
    // ensure output_dir exists
    fs::create_dir_all(&output_dir).await.unwrap();
    // write metadata-json to output_dir/metadata.json
    let metadata_path = Path::new(&output_dir).join("metadata.json");
    fs::write(metadata_path, metadata_str).await.unwrap();
    debug!("wrote metadata.json to {:?}", output_dir);

    let mut stmt_zx_distinct = c
        .prepare("SELECT DISTINCT zoom_level, tile_column FROM tiles")
        .unwrap();

    let zx_iter = stmt_zx_distinct
        .query_map([], |row| {
            let zoom_level: u8 = row.get(0)?;
            let tile_column: u32 = row.get(1)?;
            let r = (zoom_level, tile_column);
            Ok(r)
        })
        .unwrap();

    let twriter = TilesFsWriter::new(output_dir.to_string());

    let zx_stream = stream::iter(zx_iter);

    zx_stream
        .for_each_concurrent(10, |zx| async {
            let zx = zx.unwrap();
            let z = zx.0;
            let x = zx.1;
            twriter.mkdirpath(z, x).await;
        })
        .await;

    let mut stmt = c
        .prepare("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles")
        .unwrap();

    let tiles_iter = stmt
        .query_map([], |row| {
            let zoom_level: u8 = row.get(0)?;
            let tile_column: u32 = row.get(1)?;

            let tile_row: u32 = row.get(2)?;
            let tile_data: Vec<u8> = row.get(3)?;

            let r = MbtTileRow::new(zoom_level, tile_column, tile_row, tile_data);
            Ok(r)
        })
        .unwrap();

    let tiles_stream = stream::iter(tiles_iter);

    // let count = 0;
    tiles_stream
        .for_each_concurrent(0, |tile| async {
            // print smaller rep
            // println!("tile: {} {} {} {}"
            // , tile.tile_column, tile.tile_row, tile.zoom_level, tile.tile_data.len());
            // sleep for .1 seconds
            match tile {
                Ok(tile) => {
                    let t = Tile::new(tile.tile_column, tile.tile_row, tile.zoom_level);
                    twriter.write_tile(tile).await;
                    debug!("Wrote tile: {}", t);

                    // let dur2 = Duration::from_millis(1000);
                    // time::sleep(dur2).await;
                }
                Err(e) => {
                    println!("tile error: {e:?}");
                    warn!("tile error: {:?}", e);
                }
            }
            // let tile_msg = tile.json_obj();
            // let dur = Duration::from_millis(100);

            // time::sleep(dur).await;
            // twriter.write_tile(tile).await;
            //
            // if twriter.nwritten() % 1000 == 0 {
            //     println!("nwritten: {:?}", twriter.nwritten());
            //     let percent = (twriter.nwritten() as f32 / total_tiles as f32) * 100.0;
            //     // "nwritten: {:?} [{:?}]"
            //     let msg = format!("nwritten: {:?} [{:?}]", twriter.nwritten(), percent);
            //     // println!("percent: {:?}", percent);
            //     println!("{}", msg);
            // }

            // sleep for .1 seconds
            // let dur = Duration::from_millis(100);
            // time::sleep(dur).await;
            // println!("DONE tile: {:?}", tile_msg);
        })
        .await;

    let end_time = std::time::Instant::now();
    let elapsed = end_time - start_time;
    let elapsed_secs = elapsed.as_secs();
    println!("elapsed_secs: {elapsed_secs:?}");
}

pub struct CopyConfig {
    pub src: Source,
    pub dst: Destination,
}

impl CopyConfig {
    pub fn new(src: Source, dst: Destination) -> Self {
        Self { src, dst }
    }
}

pub async fn copy_main(args: CopyArgs) {
    warn!("experimental command: copy/cp");

    //let file = "D:\\utiles\\blue-marble\\blue-marble.z0z4.normal.mbtiles";
    // make sure input file exists and is file...
    let src_path = Path::new(&args.src);
    assert!(
        src_path.exists(),
        "File does not exist: {}",
        src_path.display()
    );
    assert!(
        src_path.is_file(),
        "Not a file: {filepath}",
        filepath = src_path.display()
    );

    // make sure output dir does not exist
    let dst_path = Path::new(&args.dst);
    let dst_path_exists = dst_path.exists();
    if dst_path_exists {
        if args.force {
            warn!("dst_path exists: {:?}, but force is true", dst_path);
        } else {
            assert!(!dst_path_exists, "File exists: {}", dst_path.display());
        }
    }
    let src = Source::Mbtiles(src_path.to_str().unwrap().to_string());
    let dst = Destination::Fs(dst_path.to_str().unwrap().to_string());

    let cfg = CopyConfig::new(src, dst);

    match cfg.src {
        Source::Mbtiles(filepath) => match cfg.dst {
            Destination::Fs(output_dir) => {
                copy_mbtiles2fs(filepath, output_dir).await;
            }
        },
    }
}
