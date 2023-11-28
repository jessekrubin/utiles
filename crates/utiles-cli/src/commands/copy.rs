use utilesqlite::{Mbtiles, MbtilesAsync};
// use tokio_stream::{self as stream, Stream};
use futures::stream::{self, StreamExt};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct MbtTile {
    zoom_level: u8,
    tile_column: u32,
    tile_row: u32,
    tile_data: Vec<u8>,
}

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
            .join(format!("{}", z))
            .join(format!("{}", x))
    }

    fn filepath(&self, z: u8, x: u32, y: u32) -> PathBuf {
        self.dirpath(z, x).join(format!("{}.png", y))
    }

    pub async fn mkdirpath(&self, z: u8, x: u32) {
        let dp = self.dirpath(z, x);
        let dp = dp.to_str().unwrap();
        fs::create_dir_all(dp).await.unwrap();
    }

    pub async fn write_tile(&self, tile: MbtTile) {
        let filepath = self.filepath(tile.zoom_level, tile.tile_column, tile.tile_row);
        debug!("filepath: {:?}", filepath);
        fs::write(filepath, tile.tile_data).await.unwrap();
        // increment stats
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

pub async fn copy_main() {
    let file = "D:\\maps\\reptiles\\mbtiles\\blue-marble\\blue-marble.mbtiles";
    let mbt = Mbtiles::from_filepath(file).unwrap();

    let total_tiles: u32 = mbt
        .conn()
        .query_row("SELECT count(*) FROM tiles", [], |row| row.get(0))
        .unwrap();
    println!("total_tiles: {total_tiles:?}");
    let c = mbt.conn();


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


    let mut twriter = TilesFsWriter::new(
        "D:\\utiles\\crates\\utiles-cli\\blue-marble-tiles".to_string(),
    );

    let zx_stream = stream::iter(zx_iter);
    zx_stream.for_each_concurrent(
        10,
        |zx| async {
            let zx = zx.unwrap();
            let z = zx.0;
            let x = zx.1;
            twriter.mkdirpath(z, x).await;
        },
    ).await;

    let mut stmt = c
        .prepare("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles")
        .unwrap();


    let tiles_iter = stmt
        .query_map([], |row| {
            let zoom_level: u8 = row.get(0)?;
            let tile_column: u32 = row.get(1)?;

            let tile_row: u32 = row.get(2)?;
            let tile_data: Vec<u8> = row.get(3)?;

            let r = MbtTile {
                zoom_level,
                tile_column,
                tile_row,
                tile_data,
            };
            Ok(r)
        })
        .unwrap();

    let tiles_stream = stream::iter(tiles_iter);

    // let count = 0;
    tiles_stream
        .for_each_concurrent(10, |tile| async {
            // print smaller rep
            // println!("tile: {} {} {} {}"
            // , tile.tile_column, tile.tile_row, tile.zoom_level, tile.tile_data.len());
            let tile = tile.unwrap();
            twriter.write_tile(tile).await;

            if twriter.nwritten() % 1000 == 0 {
                println!("nwritten: {:?}", twriter.nwritten());
                let percent = (twriter.nwritten() as f32 / total_tiles as f32) * 100.0;
                // "nwritten: {:?} [{:?}]"
                let msg = format!("nwritten: {:?} [{:?}]", twriter.nwritten(), percent);
                // println!("percent: {:?}", percent);
                println!("{}", msg);
            }
        })
        .await;

}
