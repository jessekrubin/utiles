#![allow(clippy::unwrap_used)]
use std::path::PathBuf;

use anyhow::Result;

use utiles::mbt::Mbtiles;
use utiles::{utile, Tile};

fn get_utiles_test_osm_mbtiles_path() -> Result<PathBuf> {
    let pwd = std::env::current_dir()?;
    let repo_root = pwd.parent()?.parent()?;
    Ok(repo_root.join("test-data/mbtiles/osm-standard.z0z4.mbtiles"))
}

fn printsep() {
    // 80 chars
    println!("{}", "-".repeat(80));
}

fn main() -> Result<()> {
    let src = get_utiles_test_osm_mbtiles_path()?;
    println!("mbtiles path: {src:?}");

    printsep();
    let mbt = Mbtiles::open_existing(src)?;
    println!("mbtiles: {mbt:?}");
    printsep();

    let metadata = mbt.metadata();
    println!("metadata: {metadata:?}");

    printsep();
    let count = mbt.tiles_count();
    println!("tiles count: {count:?}");

    printsep();
    let tile = utile!(0, 0, 0);
    let a_tile = mbt.query_tile(&tile)?;
    if let Some(tile_data) = a_tile {
        println!("tile (size): {:?}", tile_data.len());
    } else {
        println!("tile not found: {tile:?}");
    }
    Ok(())
}
