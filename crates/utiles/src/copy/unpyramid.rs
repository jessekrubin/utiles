use std::path::Path;

use tokio::sync::mpsc;
use tokio::{fs, task};
use tracing::{debug, warn};
use walkdir::WalkDir;

use utiles_core::tile_data_row::TileData;
use utiles_core::Tile;

use crate::copy::CopyConfig;
use crate::errors::UtilesResult;
use crate::mbt::parse_metadata_json;
use crate::utilesqlite::Mbtiles;

fn fspath2xyz(path: &Path) -> UtilesResult<(u32, u32, u8)> {
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

pub async fn copy_fs2mbtiles(cfg: &CopyConfig) -> UtilesResult<()> {
    let dirpath = &cfg.src;
    let mbtiles_path = &cfg.dst;
    debug!("dirpath: {dirpath:?}, mbtiles: {mbtiles_path:?} cfg: {cfg:?}");
    let metadata_path = Path::new(&dirpath).join("metadata.json");
    let walker = WalkDir::new(&dirpath).min_depth(3).max_depth(3);
    let mut dst_mbt = Mbtiles::open(&mbtiles_path)?;
    dst_mbt
        .init_flat_mbtiles()
        .expect("init_flat_mbtiles failed");

    if let Ok(metadata_str) = fs::read_to_string(metadata_path).await {
        // found metadata.json
        let metadata_vec = parse_metadata_json(&metadata_str);
        match metadata_vec {
            Ok(metadata_vec) => {
                dst_mbt.metadata_set_from_vec(&metadata_vec)?;
            }
            Err(e) => {
                warn!("e: {e:?}");
            }
        }
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
    Ok(())
}
