use anyhow::Result;
use std::path::PathBuf;
use utiles::mbt::{MbtilesAsync, MbtilesClientAsync};
use utiles_core::utile;
use utiles_core::Tile;
fn get_utiles_test_osm_mbtiles_path() -> PathBuf {
    let pwd = std::env::current_dir().unwrap();
    let repo_root = pwd.parent().unwrap().parent().unwrap();
    repo_root.join("test-data/mbtiles/osm-standard.z0z4.mbtiles")
}

fn printsep() {
    // 80 chars
    println!("{}", "-".repeat(80));
}

#[tokio::main]
async fn main() -> Result<()> {
    let src = get_utiles_test_osm_mbtiles_path();
    println!("mbtiles path: {:?}", src);

    printsep();
    let mbt = MbtilesClientAsync::open_existing(src).await?; // .await
    println!("mbtiles: {:?}", mbt);

    printsep();
    let metadata = mbt.metadata_rows().await?;
    println!("metadata: {:?}", metadata);

    printsep();
    let count = mbt.tiles_count().await?;
    println!("tiles count: {:?}", count);

    printsep();
    let tile = utile!(0, 0, 0);
    let a_tile = mbt.query_tile(&tile).await?;
    if let Some(tile_data) = a_tile {
        println!("tile (size): {:?}", tile_data.len());
    } else {
        println!("tile not found: {:?}", tile);
    }

    Ok(())
}
