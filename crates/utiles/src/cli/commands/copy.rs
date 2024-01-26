use std::cell::Cell;
use std::path::{Path, PathBuf};

use futures::stream::{self, StreamExt};
use serde_json;
use tokio::fs;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use utiles_core::bbox::BBox;
use utiles_core::mbutiles::{MbtTileRow, MbtilesMetadataRow};
use utiles_core::tile_data_row::TileData;
use utiles_core::{tile_ranges, Tile, TileLike};

use crate::cli::args::CopyArgs;
use crate::utilesqlite::Mbtiles;

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
        // HERE YOU NEED TO FLIP THE THING JESSE
        let filepath = self.dirpath(tile.z(), tile.x()).join(format!(
            "{}.{}",
            // flipy(tile.y(), tile.z()),
            tile.y(),
            tile.extension()
        ));
        // debug!("filepath: {:?}", filepath);
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

#[derive(Debug)]
pub enum Source {
    Mbtiles(String),
    Fs(String),
}

#[derive(Debug)]
pub enum Destination {
    Mbtiles(String),
    Fs(String),
}

#[derive(Debug)]
pub struct CopyConfig {
    pub src: Source,
    pub dst: Destination,

    pub zooms: Option<Vec<u8>>,
    pub bbox: Option<BBox>,
}

pub enum CopySrcDest {
    Mbtiles2Fs,
    Fs2Mbtiles,
}

impl CopyConfig {
    pub fn new(
        src: Source,
        dst: Destination,
        zooms: Option<Vec<u8>>,
        bbox: Option<BBox>,
    ) -> Self {
        Self {
            src,
            dst,
            zooms,
            bbox,
        }
    }

    // pub fn sql_where_for_zoom(&self, zoom: u8) -> String {
    //     let pred = match &self.bbox {
    //         Some(bbox) => {
    //             let trange = tile_ranges(bbox.tuple(), vec![zoom].into());
    //             trange.sql_where(Some(true))
    //         }
    //         None => {
    //             format!("zoom_level = {zoom}")
    //         }
    //     };
    //     // attach 'WHERE'
    //     if pred.is_empty() {
    //         pred
    //     } else {
    //         format!("WHERE {pred}")
    //     }
    // }

    pub fn mbtiles_sql_where(&self, zoom_levels: Option<Vec<u8>>) -> String {
        let pred = match (&self.bbox, &self.zooms) {
            (Some(bbox), Some(zooms)) => {
                let trange = tile_ranges(
                    bbox.tuple(),
                    zoom_levels.unwrap_or(zooms.clone()).into(),
                );
                trange.mbtiles_sql_where()
            }
            (Some(bbox), None) => {
                let trange = tile_ranges(
                    bbox.tuple(),
                    zoom_levels
                        .unwrap_or((0..28).map(|z| z as u8).collect::<Vec<u8>>())
                        .into(),
                );
                trange.mbtiles_sql_where()
            }
            (None, Some(zooms)) => {
                format!(
                    "zoom_level IN ({zooms})",
                    zooms = zooms
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            (None, None) => String::new(),
        };
        // attach 'WHERE'
        if pred.is_empty() {
            pred
        } else {
            format!("WHERE {pred}")
        }
    }
}

async fn copy_mbtiles2fs(mbtiles: String, output_dir: String, cfg: CopyConfig) {
    let mbt = Mbtiles::from(mbtiles.as_ref());

    let where_clause = cfg.mbtiles_sql_where(
        // Some(zoom_levels_for_where)
        // flip happens here maybe
        None,
    );
    let start_time = std::time::Instant::now();

    let count_query = &"SELECT count(*) FROM tiles".to_string();
    let total_tiles: u32 = mbt
        .conn()
        .query_row(count_query, [], |row| row.get(0))
        .unwrap();

    debug!("total_tiles: {:?}", total_tiles);

    info!("# tiles: {total_tiles:?} ~ {mbtiles:?} => {output_dir:?}");
    let c = mbt.conn();

    let res_metadata_vec = mbt.metadata();
    let metadata_vec = res_metadata_vec.unwrap_or_else(|e| {
        warn!("e: {e:?}");
        vec![]
    });
    let metadata_str = serde_json::to_string_pretty(&metadata_vec).unwrap();
    // ensure output_dir exists
    fs::create_dir_all(&output_dir).await.unwrap();
    // write metadata-json to output_dir/metadata.json
    let metadata_path = Path::new(&output_dir).join("metadata.json");
    fs::write(metadata_path, metadata_str).await.unwrap();
    debug!("wrote metadata.json to {:?}", output_dir);

    let mut stmt_zx_distinct = c
        .prepare(
            format!(
                "SELECT DISTINCT zoom_level, tile_column FROM tiles {where_clause}"
            )
            .as_str(),
        )
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

    // let mut stmt = c
    //     .prepare("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles")
    //     .unwrap();
    let tiles_query = format!(
        "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles {where_clause}"
    );

    debug!("tiles_query: {:?}", tiles_query);
    let mut stmt = c.prepare(tiles_query.as_str()).unwrap();

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
            match tile {
                Ok(tile) => {
                    twriter.write_tile(tile).await;
                }
                Err(e) => {
                    warn!("tile error: {:?}", e);
                }
            }
        })
        .await;

    let end_time = std::time::Instant::now();
    let elapsed = end_time - start_time;
    let elapsed_secs = elapsed.as_secs();
    debug!("elapsed_secs: {elapsed_secs:?}");
}

fn fspath2xyz(path: &Path) -> Result<(u32, u32, u8), std::num::ParseIntError> {
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

async fn copy_fs2mbtiles(dirpath: String, mbtiles: String, _cfg: CopyConfig) {
    let metadata_path = Path::new(&dirpath).join("metadata.json");
    let walker = WalkDir::new(&dirpath).min_depth(3).max_depth(3);
    let mut dst_mbt = Mbtiles::open(&mbtiles).unwrap();
    dst_mbt
        .init_flat_mbtiles()
        .expect("init_flat_mbtiles failed");
    // let c = dst_mbt.conn.trace(Some(|s| {
    //     debug!("SQL: {:?}", s);
    // }));
    // Write metadata to db if exists...

    if let Ok(metadata_str) = fs::read_to_string(metadata_path).await {
        let metadata_vec: Vec<MbtilesMetadataRow> =
            serde_json::from_str(&metadata_str).unwrap();
        dst_mbt.metadata_set_from_vec(&metadata_vec).unwrap();
    }

    let (tx, mut rx) = mpsc::channel(64);

    // Database insertion task
    let db_task = task::spawn(async move {
        let mut tiles = Vec::with_capacity(999);
        let mut nwritten = 0;
        while let Some(tile_data) = rx.recv().await {
            tiles.push(tile_data);
            if tiles.len() >= 999 {
                debug!("inserting tiles: {:?}", tiles.len());
                let n_affected = dst_mbt
                    .insert_tiles_flat(&tiles)
                    .expect("insert tiles flat failed");
                nwritten += n_affected;
                tiles.clear();
            }
        }
        // Insert any remaining tiles
        if !tiles.is_empty() {
            let n_affected = dst_mbt
                .insert_tiles_flat(&tiles)
                .expect("insert tiles flat failed");
            nwritten += n_affected;
        }
        debug!("nwritten: {:?}", nwritten);
    });

    // File processing tasks
    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path().to_owned();
        let tx_clone = tx.clone();
        let tile_xyz = fspath2xyz(&path);
        match tile_xyz {
            Ok(tile_xyz) => {
                task::spawn(async move {
                    let data = fs::read(&path).await.unwrap();
                    let tile_data = TileData::new(
                        Tile::new(tile_xyz.0, tile_xyz.1, tile_xyz.2),
                        data,
                    );
                    tx_clone.send(tile_data).await.unwrap();
                    debug!("sent tile: {:?}", tile_xyz);
                });
            }
            Err(e) => {
                warn!("e: {e:?}");
            }
        }
    }
    debug!("dropping tx");
    // drop tx to close the channel
    drop(tx);
    // Wait for the database task to complete
    db_task.await.unwrap();
}

#[allow(dead_code)]
async fn copy_fs2mbtiles_simple(dirpath: String, mbtiles: String) {
    let metadata_path = Path::new(&dirpath).join("metadata.json");
    let batch_size = 2048; // Define your batch size
                           // get all files...
    let walker = WalkDir::new(dirpath).min_depth(3).max_depth(3);
    let mut dst_mbt = Mbtiles::open(&mbtiles).unwrap();

    dst_mbt
        .init_flat_mbtiles()
        .expect("init_flat_mbtiles failed");
    let mut tiles: Vec<TileData> = vec![];
    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        debug!("path_str: {:?}", path_str);
        let t2 = fspath2xyz(path);
        match t2 {
            Ok(t2) => {
                debug!("t2: {:?}", t2);
                let data = fs::read(path).await.unwrap();
                let tile = Tile::new(t2.0, t2.1, t2.2);

                // sleep for a 0.1 second
                let dur = Duration::from_millis(100);
                sleep(dur).await;

                // insert tile
                let tdata = TileData::new(tile, data);
                tiles.push(tdata);
                if tiles.len() > batch_size {
                    debug!("inserting tiles: {:?}", tiles.len());
                    let naff = dst_mbt
                        .insert_tiles_flat(&tiles)
                        .expect("insert tiles flat failed");
                    debug!("naff: {naff:?}");
                    // dst_mbt.insert_tiles_flat(tiles).await.unwrap();
                    tiles = vec![];
                }
            }
            Err(e) => {
                warn!("e: {e:?}");
            }
        }
    }
    if !tiles.is_empty() {
        let naff = dst_mbt
            .insert_tiles_flat(&tiles)
            .expect("insert tiles flat failed");
        debug!("Number of inserts: {naff:?}");
    }

    // if DIR/metadata.json exists we set the metadata from it
    if let Ok(metadata_str) = fs::read_to_string(metadata_path).await {
        let metadata_vec: Vec<MbtilesMetadataRow> =
            serde_json::from_str(&metadata_str).unwrap();
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
            panic!("src is not file or dir: {src_path:?}");
        }
    } else {
        panic!("src does not exist: {src_path:?}");
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
    // match args.zoom {
    //     Some(zoom) => {
    //         info!("zoom: {:?}", zoom);
    //     }
    //     None => {
    //         info!("no zoom");
    //     }
    // }
    let zooms: Option<Vec<u8>> = args.zooms();
    let bbox = args.bbox;

    let cfg = CopyConfig::new(
        get_tile_src(&args.src),
        get_tile_dst(&args.dst),
        zooms,
        bbox,
    );

    // log it out
    debug!("cfg: {:?}", cfg);
    // make sure input file exists and is file...
    let src = get_tile_src(&args.src);
    let dst = get_tile_dst(&args.dst);

    let srcdst = match (src, dst) {
        (Source::Mbtiles(_src), Destination::Fs(_dst)) => CopySrcDest::Mbtiles2Fs,
        (Source::Fs(_src), Destination::Mbtiles(_dst)) => CopySrcDest::Fs2Mbtiles,
        _ => panic!("src/dst combo not supported"),
    };
    match srcdst {
        CopySrcDest::Mbtiles2Fs => {
            copy_mbtiles2fs(args.src, args.dst, cfg).await;
        }
        CopySrcDest::Fs2Mbtiles => {
            copy_fs2mbtiles(args.src, args.dst, cfg).await;
        }
    }
}
