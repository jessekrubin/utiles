use std::cell::Cell;
use std::path::{Path, PathBuf};

use crate::mbt::{metadata2map_val, metadata_vec_has_duplicates};
use futures::stream::{self, StreamExt};
use tokio::fs;
use tracing::{debug, info, warn};

use utiles_core::TileLike;

use crate::copy::CopyConfig;
use crate::errors::{UtilesError, UtilesResult};
use crate::mbt::MbtTileRow;
use crate::utilesqlite::Mbtiles;

#[derive(Debug)]
pub struct WriterStats {
    pub nwritten: Cell<u32>,
}

#[derive(Debug)]
pub struct TilePyramidFsWriter {
    root_dirpath: PathBuf,
    stats: WriterStats,
}

impl TilePyramidFsWriter {
    pub fn new(root_dirpath: PathBuf) -> Self {
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

    pub async fn mkdirpath(&self, z: u8, x: u32) -> UtilesResult<()> {
        let dp = self.dirpath(z, x);
        match dp.to_str() {
            Some(dp) => {
                fs::create_dir_all(dp).await?;
                Ok(())
            }
            None => Err(UtilesError::PathConversionError(format!(
                "path conversion error: {dp:?}"
            ))),
        }
    }

    pub async fn write_tile(&self, tile: MbtTileRow) -> UtilesResult<()> {
        // HERE YOU NEED TO FLIP THE THING JESSE
        let filepath = self.dirpath(tile.z(), tile.x()).join(format!(
            "{}.{}",
            // flipy(tile.y(), tile.z()),
            tile.y(),
            tile.extension()
        ));
        fs::write(filepath, tile.tile_data).await?;
        self.inc_nwritten();
        Ok(())
    }

    pub fn inc_nwritten(&self) {
        let n = self.stats.nwritten.get();
        self.stats.nwritten.set(n + 1);
    }
}

pub async fn copy_mbtiles2fs(cfg: &CopyConfig) -> UtilesResult<()> {
    let mbt_path = Path::new(&cfg.src);
    let output_dir = Path::new(&cfg.dst);
    let mbt = Mbtiles::from(mbt_path);

    let where_clause = cfg.mbtiles_sql_where()?;
    info!("where_clause: {where_clause:?}");
    let start_time = std::time::Instant::now();
    let count_query = &"SELECT count(*) FROM tiles".to_string();
    let total_tiles: u32 = mbt.conn().query_row(count_query, [], |row| row.get(0))?;
    debug!("total_tiles: {:?}", total_tiles);
    info!("# tiles: {total_tiles:?} ~ {mbt_path:?} => {output_dir:?}");
    let c = mbt.conn();
    let res_metadata_vec = mbt.metadata();
    let metadata_vec = res_metadata_vec.unwrap_or_else(|e| {
        warn!("e: {e:?}");
        vec![]
    });
    let metadata_str = if metadata_vec_has_duplicates(&metadata_vec) {
        warn!("metadata has duplicates writing as array...");
        serde_json::to_string_pretty(&metadata_vec)?
    } else {
        let metadata_obj = metadata2map_val(&metadata_vec);
        serde_json::to_string_pretty(&metadata_obj)?
    };
    // serde_json::to_string_pretty(&metadata_vec)?;
    // ensure output_dir exists
    fs::create_dir_all(&output_dir).await?;
    // write metadata-json to output_dir/metadata.json
    let metadata_path = Path::new(&output_dir).join("metadata.json");
    fs::write(metadata_path, metadata_str).await?;
    debug!("wrote metadata.json to {:?}", output_dir);

    let mut stmt_zx_distinct = c.prepare(
        format!("SELECT DISTINCT zoom_level, tile_column FROM tiles {where_clause}")
            .as_str(),
    )?;

    let zx_iter = stmt_zx_distinct.query_map([], |row| {
        let zoom_level: u8 = row.get(0)?;
        let tile_column: u32 = row.get(1)?;
        let r = (zoom_level, tile_column);
        Ok(r)
    })?;

    let twriter = TilePyramidFsWriter::new(output_dir.into());

    let zx_stream = stream::iter(zx_iter);

    zx_stream
        .for_each_concurrent(Some(cfg.njobs().into()), |zx| async {
            let zx = zx;
            match zx {
                Ok(zx) => {
                    let z = zx.0;
                    let x = zx.1;
                    match twriter.mkdirpath(z, x).await {
                        Ok(()) => {}
                        Err(e) => {
                            warn!("mkdirpath error: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("zx error: {:?}", e);
                }
            }
        })
        .await;
    let tiles_query = format!(
        "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles {where_clause}"
    );

    debug!("tiles_query: {:?}", tiles_query);
    let mut stmt = c.prepare(tiles_query.as_str())?;
    let tiles_iter = stmt.query_map([], |row| {
        let zoom_level: u8 = row.get(0)?;
        let tile_column: u32 = row.get(1)?;
        let tile_row: u32 = row.get(2)?;
        let tile_data: Vec<u8> = row.get(3)?;
        let r = MbtTileRow::new(zoom_level, tile_column, tile_row, tile_data);
        Ok(r)
    })?;

    let tiles_stream = stream::iter(tiles_iter);

    // let count = 0;
    tiles_stream
        .for_each_concurrent(0, |tile| async {
            match tile {
                Ok(tile) => match twriter.write_tile(tile).await {
                    Ok(()) => {}
                    Err(e) => {
                        warn!("tile error: {:?}", e);
                    }
                },
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
    Ok(())
}
