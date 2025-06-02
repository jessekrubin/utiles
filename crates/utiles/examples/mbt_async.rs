use anyhow::Result;
use futures::StreamExt;
use std::path::PathBuf;
use utiles::mbt::{MbtilesAsync, MbtilesClientAsync};
use utiles::{utile, Tile, TileLike};

fn get_utiles_test_osm_mbtiles_path() -> Result<PathBuf> {
    let pwd = std::env::current_dir()?;
    let repo_root = pwd
        .parent()
        .ok_or(anyhow::anyhow!("repo root not found"))?
        .parent()
        .ok_or(anyhow::anyhow!("repo root not found"))?;
    Ok(repo_root.join("test-data/mbtiles/osm-standard.z0z4.mbtiles"))
}

fn printsep() {
    // 80 chars
    println!("{}", "-".repeat(80));
}

#[tokio::main]
async fn main() -> Result<()> {
    let src = get_utiles_test_osm_mbtiles_path()?;
    println!("mbtiles path: {}", src.display());

    printsep();
    let mbt = MbtilesClientAsync::open_existing(src).await?;
    println!("mbtiles: {mbt:?}");

    printsep();
    let metadata = mbt.metadata_rows().await?;
    println!("metadata: {metadata:?}");

    printsep();
    let count = mbt.tiles_count().await?;
    println!("tiles count: {count:?}");

    printsep();
    let tile = utile!(0, 0, 0);
    let a_tile = mbt.query_tile(&tile).await?;
    if let Some(tile_data) = a_tile {
        println!("tile (size): {:?}", tile_data.len());
    } else {
        println!("tile not found: {tile:?}");
    }

    // stream over tiles
    let mut stream = mbt.tiles_stream(None)?;
    let mut count = 0;
    while let Some((tile, tile_data)) = stream.next().await {
        println!("tile: {} ~ size: {}", tile.json_arr(), tile_data.len());
        count += 1;
    }
    println!("tiles count: {count:?}");
    Ok(())
}
