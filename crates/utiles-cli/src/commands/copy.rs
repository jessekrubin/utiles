use std::cell::Cell;
use std::path::{Path, PathBuf};

use futures::stream::{self, StreamExt};
use serde_json;
use tokio::fs;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use utiles::mbtiles::{MbtilesMetadataRow, MbtTileRow};
use utiles::tile_data_row::TileData;
use utiles::{flipy, Tile, TileLike};
use utilesqlite::{Mbtiles};
use crate::args::CopyArgs;

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

    // pub fn nwritten(&self) -> u32 {
    //     self.stats.nwritten.get()
    // }
}

pub enum Source {
    Mbtiles(String),
    Fs(String),
}

pub enum Destination {
    Mbtiles(String),
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
    println!("{metadata_str}");
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

pub enum CopySrcDest {
    Mbtiles2Fs,
    Fs2Mbtiles,
}

impl CopyConfig {
    pub fn new(src: Source, dst: Destination) -> Self {
        Self { src, dst }
    }
}

// fn fspath2tile(fspath: &Path) -> Option<Tile> {
//     let parts = fspath.
//     let parts: Vec<&str> = fspath.split('/').collect();
//     if parts.len() <3 {
//         return None;
//     }
//
//     let z =  match parts[parts.len()- 3].parse::<u8>() {
//         Ok(z) => z,
//         Err(e) => {
//             println!("e: {:?}", e);
//             return None;
//         }
//     };
//     let x = match parts[parts.len() - 2].parse::<u32>() {
//         Ok(x) => x,
//         Err(e) => {
//             println!("e: {:?}", e);
//             return None;
//         }
//     };
//     let y = match parts[parts.len() - 1].split('.').next().unwrap().parse::<u32>() {
//         Ok(y) => y,
//         Err(e) => {
//             println!("e: {:?}", e);
//             return None;
//         }
//     };
//     Some(Tile::new(x, y, z))
// }

fn extract_xyz_from_path(
    path: &Path,
) -> Result<(u32, u32, u8), std::num::ParseIntError> {
    let path = Path::new(path);
    let mut components = path.components().rev();

    let y_with_ext = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    let y = y_with_ext.split('.').next().unwrap_or("").parse::<u32>()?;

    let x = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("")
        .parse::<u32>()?;
    let z = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("")
        .parse::<u8>()?;

    Ok((x, y, z))
}

// async fn copy_fs2mbtiles2(dirpath: String, mbtiles: String) {
//     let new_mbta = MbtilesAsync::new_flat_mbtiles(&mbtiles).await.unwrap();
//     let batch_size = 1000; // Define your batch size
//
//     // Create an async channel
//     let (tx, mut rx) = mpsc::channel(32); // Adjust the channel size as needed
//
//     // Database insertion task
//     let db_task = tokio::spawn(async move {
//         let mut tiles_batch = Vec::with_capacity(batch_size);
//         while let Some(tile_data) = rx.recv().await {
//             tiles_batch.push(tile_data);
//             if tiles_batch.len() >= batch_size {
//                 new_mbta.insert_tiles_flat(tiles_batch.clone()).await.unwrap();
//                 tiles_batch.clear();
//             }
//         }
//         // Insert any remaining tiles in the batch
//         if !tiles_batch.is_empty() {
//             new_mbta.insert_tiles_flat(tiles_batch).await.unwrap();
//         }
//     });
//
//     // File processing tasks
//     let walker_stream = stream::iter(
//         WalkDir::new(dirpath)
//             .min_depth(3)
//             .max_depth(3)
//             .into_iter()
//             .filter_map(|e| {
//                 let e = e.ok()?;
//                 if e.file_type().is_file() {
//                     Some(e)
//                 } else {
//                     None
//                 }
//             })
//             .map(|entry| entry.path().to_owned())
//     );
//
//
//     walker_stream
//         .for_each_concurrent(/* limit parallelism here */2, move |path| {
//             let tx = tx.clone(); // Clone the sender for each task
//             async move {
//                 if let Ok(t2) = extract_xyz_from_path(&path) {
//                     if let Ok(data) = fs::read(&path).await {
//                         let tile = Tile::new(t2.0, t2.1, t2.2);
//                         let tdata = TileData::new(tile, data);
//                         tx.send(tdata).await.unwrap(); // Send to the channel
//                     }
//                 }
//             }
//         })
//         .await;
//
//     let futures = walker_stream.map(|path| {
//         let tx = tx.clone();
//         async move {
//             if let Ok(t2) = extract_xyz_from_path(&path) {
//                 if let Ok(data) = fs::read(&path).await {
//                     let tile = Tile::new(t2.0, t2.1, t2.2);
//                     let tdata = TileData::new(tile, data);
//                     tx.send(tdata).await.unwrap();
//                 }
//             }
//         }
//     }).collect::<Vec<_>>();
//
//     // Wait for all file processing tasks to complete
//     join_all(futures).await;
//
//     // Close the channel
//     drop(tx);
//
//     // Await the database task to complete
//     db_task.await.unwrap();
// }

async fn copy_fs2mbtiles(dirpath: String, mbtiles: String) {
    let metadata_path = Path::new(&dirpath).join("metadata.json");
    let batch_size = 2048; // Define your batch size
                           // get all files...
    let walker = WalkDir::new(dirpath).min_depth(3).max_depth(3);
    // let dst_mbt = MbtilesAsync::new_flat_mbtiles(&mbtiles).await.unwrap();
    let mut dst_mbt = Mbtiles::open(&mbtiles).unwrap();

    dst_mbt
        .init_flat_mbtiles()
        .expect("init_flat_mbtiles failed");

    // {
    //
    //     let c = new_mbta.pool.get().await.unwrap();
    //     let r = c
    //         .interact(|conn| {
    //         //     get the current config
    //         })
    //         .await;
    // }
    //
    let mut tiles: Vec<TileData> = vec![];
    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        debug!("path_str: {:?}", path_str);
        // println!("path_str: {:?}", path_str);
        // println!("tiles len: {:?}", tiles.len());
        let t2 = extract_xyz_from_path(path);
        match t2 {
            Ok(t2) => {
                debug!("t2: {:?}", t2);

                let data = fs::read(path).await.unwrap();
                let tile = Tile::new(t2.0, t2.1, t2.2);
                // insert tile
                let tdata = TileData::new(tile, data);
                tiles.push(tdata);
                if tiles.len() > batch_size {
                    println!("inserting tiles: {:?}", tiles.len());
                    let naff = dst_mbt
                        .insert_tiles_flat(tiles)
                        .expect("insert tiles flat failed");
                    println!("naff: {:?}", naff);
                    // dst_mbt.insert_tiles_flat(tiles).await.unwrap();
                    tiles = vec![];
                }
            }
            Err(e) => {
                println!("e: {:?}", e);
            }
        }
    }

    if !tiles.is_empty() {
        println!("inserting tiles: {:?}", tiles.len());
        let naff = dst_mbt
            .insert_tiles_flat(tiles)
            .expect("insert tiles flat failed");
        println!("naff: {:?}", naff);
        // dst_mbt.insert_tiles_flat(tiles).await.unwrap();
        tiles = vec![];
    }

   // if DIR/metadata.json exists we set the metadata from it
    if let Ok(metadata_str) = fs::read_to_string(
        metadata_path
    ).await {
        let metadata_vec: Vec<MbtilesMetadataRow> = serde_json::from_str(&metadata_str).unwrap();
        dst_mbt.metadata_set_from_vec(&metadata_vec).unwrap();
    }
}


fn get_tile_src(src: &str) -> Source {
    let src_path = Path::new(src);
    if src_path.exists() {
        if src_path.is_file() {
            Source::Mbtiles(src.to_string())
        } else if src_path.is_dir() {
            Source::Fs(src.to_string())
        } else {
            panic!("src is not file or dir: {:?}", src_path);
        }
    } else {
        panic!("src does not exist: {:?}", src_path);
    }
}

fn get_tile_dst(dst: &str) -> Destination {
    // if it contains '.mbtiles' then it's a mbtiles file
    // else it's a directory
    if dst.contains(".mbtiles") {
        Destination::Mbtiles(dst.to_string())
    } else {
        Destination::Fs(dst.to_string())
    }
}

pub async fn copy_main(args: CopyArgs) {
    warn!("experimental command: copy/cp");

    //let file = "D:\\utiles\\blue-marble\\blue-marble.z0z4.normal.mbtiles";
    // make sure input file exists and is file...
    let src = get_tile_src(&args.src);
    let dst = get_tile_dst(&args.dst);

    let srcdst = match (src, dst) {
        (Source::Mbtiles(src), Destination::Fs(dst)) => CopySrcDest::Mbtiles2Fs,
        (Source::Fs(src), Destination::Mbtiles(dst)) => CopySrcDest::Fs2Mbtiles,
        _ => panic!("src/dst combo not supported"),
    };
    match srcdst {
        CopySrcDest::Mbtiles2Fs => {
            copy_mbtiles2fs(args.src, args.dst).await;
        }
        CopySrcDest::Fs2Mbtiles => {
            copy_fs2mbtiles(args.src, args.dst).await;
        }
    }

    // let src_path = Path::new(&args.src);
    // assert!(
    //     src_path.exists(),
    //     "src does not exist: {}",
    //     src_path.display()
    // );
    // assert!(
    //     src_path.is_file(),
    //     "Not a file: {filepath}",
    //     filepath = src_path.display()
    // );
    //
    // // make sure output dir does not exist
    // let dst_path = Path::new(&args.dst);
    // let dst_path_exists = dst_path.exists();
    // if dst_path_exists {
    //     if args.force {
    //         warn!("dst_path exists: {:?}, but force is true", dst_path);
    //     } else {
    //         assert!(!dst_path_exists, "File exists: {}", dst_path.display());
    //     }
    // }
    // let src = Source::Mbtiles(src_path.to_str().unwrap().to_string());
    // let dst = Destination::Fs(dst_path.to_str().unwrap().to_string());
    //
    //

    // let cfg = CopyConfig::new(src, dst);

    // match cfg.src {
    //     Source::Mbtiles(filepath) => match cfg.dst {
    //         Destination::Fs(output_dir) => {
    //             copy_mbtiles2fs(filepath, output_dir).await;
    //         }
    //     },
    // }
}
