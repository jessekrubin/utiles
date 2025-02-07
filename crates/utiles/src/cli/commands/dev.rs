use clap::Parser;
use tracing::{debug, info, warn};

use crate::errors::UtilesResult;

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

// fn _timing_agg_tiles_hash(filepath: &str) -> UtilesResult<()> {
//     let mbt = Mbtiles::open(filepath)?;
//     add_functions(&mbt.conn)?;
//     let hashes = vec![
//         HashType::Xxh3_128,
//         HashType::Xxh3_64,
//         HashType::Xxh64,
//         HashType::Xxh32,
//         HashType::Fnv1a,
//         HashType::Md5,
//     ];
//     for hash in hashes {
//         let start_time = std::time::Instant::now();
//         let agg_tile_hash = mbt_agg_tiles_hash_stream(&mbt.conn, hash, None, &None)?;
//         let elapsed = start_time.elapsed();
//         debug!("---------------------");
//         debug!("hash: {:?}, agg_tile_hash: {:?}", hash, agg_tile_hash);
//         debug!("agg_tile_hash: {:?}", agg_tile_hash);
//         debug!("elapsed: {:?}", elapsed);
//     }
//     Ok(())
// }
//
#[allow(clippy::unused_async)]
async fn dev(args: DevArgs) -> UtilesResult<()> {
    // DEV START
    debug!("args: {:?}", args);
    match args.fspath {
        Some(filepath) => {
            info!("fspath: {:?}", filepath);
            // timing_agg_tiles_hash(&filepath)?;
        }
        None => {
            warn!("no fspath provided");
        }
    }
    // DEV END
    Ok(())
}

pub(crate) async fn dev_main(args: DevArgs) -> UtilesResult<()> {
    warn!("__DEV_MAIN__");
    dev(args).await?;
    Ok(())
}
