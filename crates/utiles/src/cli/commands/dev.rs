use clap::Parser;
use tracing::{debug, warn};

use crate::utilesqlite::hash_types::HashType;
use crate::utilesqlite::Mbtiles;
use crate::utilesqlite::mbtiles::{add_functions, mbt_agg_tiles_hash};

/// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗      ██████╗ ███████╗██╗   ██╗
/// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝      ██╔══██╗██╔════╝██║   ██║
/// ██║   ██║   ██║   ██║██║     █████╗  ███████╗█████╗██║  ██║█████╗  ██║   ██║
/// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║╚════╝██║  ██║██╔══╝  ╚██╗ ██╔╝
/// ╚██████╔╝   ██║   ██║███████╗███████╗███████║      ██████╔╝███████╗ ╚████╔╝
///  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝      ╚═════╝ ╚══════╝  ╚═══╝
#[derive(Debug, Parser)]
#[command(name = "dev", about = "dev", long_about = "development/playground")]
pub struct DevArgs {
    #[arg(required = false)]
    fspath: Option<String>,
}

#[allow(clippy::unused_async)]
async fn dev(args: DevArgs) -> Result<(), Box<dyn std::error::Error>> {
    // DEV START
    debug!("args: {:?}", args);
    let filepath = "D:\\blue-marble\\blue-marble.mbtiles".to_string();
    // let filepath = "D:\\blue-marble\\blue-marble.z0z4.mbtiles".to_string();

    let mbt = Mbtiles::open(filepath)?;
    add_functions(&mbt.conn)?;

    let hashes = vec![
        HashType::Xxh3_128,
        HashType::Xxh3_64,
        HashType::Xxh64,
        HashType::Xxh32,
        HashType::Fnv1a,
        HashType::Md5,
    ];

    for hash in hashes {
        let start_time = std::time::Instant::now();
        let agg_tile_hash = mbt_agg_tiles_hash(&mbt.conn, hash)?;
        let elapsed = start_time.elapsed();
        debug!("---------------------");
        debug!("hash: {:?}, agg_tile_hash: {:?}", hash, agg_tile_hash);
        debug!("agg_tile_hash: {:?}", agg_tile_hash);
        debug!("elapsed: {:?}", elapsed);
    }

    // DEV END
    Ok(())
}

pub async fn dev_main(args: DevArgs) -> Result<(), Box<dyn std::error::Error>> {
    warn!("__DEV_MAIN__");
    dev(args).await?;
    Ok(())
}
