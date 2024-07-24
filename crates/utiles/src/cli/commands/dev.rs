use clap::Parser;
use futures::StreamExt;
use hex::ToHex;
use md5::{Digest, Md5};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};

use utiles_core::{Tile, TileLike};

use crate::errors::UtilesResult;
use crate::mbt::hash_types::HashType;
use crate::mbt::{make_tiles_stream, mbt_agg_tiles_hash_stream};
use crate::utilesqlite::mbtiles::add_functions;
use crate::utilesqlite::{Mbtiles, MbtilesAsyncSqliteClient};
use crate::UtilesError;

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
/// This function takes any Digest type, and returns a hex-encoded string.
pub async fn hash_stream<T: Digest>(mut data: ReceiverStream<Vec<u8>>) -> String {
    // Note that some hashers provide seed value set to 0 by default.
    // Use `...::from_hasher(hasher)` function to instantiate them.
    let mut hasher = T::new();
    while let Some(chunk) = data.next().await {
        hasher.update(&chunk);
    }

    // hasher.update(data);
    hasher.finalize().to_vec().encode_hex_upper()
}

pub async fn tile_stream_to_bytes_stream(
    mut data: ReceiverStream<(Tile, Vec<u8>)>,
) -> ReceiverStream<Vec<u8>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        while let Some((tile, tile_data)) = data.next().await {
            let bytes = vec![
                tile.z().to_string().as_bytes().to_vec(),
                tile.x().to_string().as_bytes().to_vec(),
                tile.yup().to_string().as_bytes().to_vec(),
                tile_data,
            ]
            .concat();
            tx.send(bytes).await.unwrap();
        }
    });
    ReceiverStream::new(rx)
}
async fn agg_hash_stream(filepath: &str) -> UtilesResult<()> {
    let mbt = MbtilesAsyncSqliteClient::open_readonly(filepath).await?;
    let mut stream = make_tiles_stream(
        &mbt,
        Some("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles ORDER BY zoom_level, tile_column, tile_row;"),
    )?;
    let mut hasher = Md5::new();
    let start_time = std::time::Instant::now();
    let bstream = tile_stream_to_bytes_stream(stream).await;
    let hash_stream = hash_stream::<Md5>(bstream);
    //
    // let (tx, rx) = tokio::sync::mpsc::channel(100);
    // let bstream = ReceiverStream::new(rx);
    // let hash_stream = hash_stream::<Md5>(
    //     bstream,
    // // );
    //
    // let bin_stream_task = tokio::spawn(
    //     || async move {
    //         ()
    //         );
    //         // drop here
    //         // drop(tx);
    //
    //         // let result = hasher.finalize();
    //         let (bin_stream_result, hash_stream_res) = tokio::join!(
    //     bin_stream_task,
    //     hash_stream,
    // );
    let res_hex = hash_stream.await;
    // let res_hex = hash_stream.await;
    let elapsed = start_time.elapsed();
    // let res_hex = format!("{:X}", result);
    println!("md5: {:?}", res_hex);
    println!("elapsed: {:?}", elapsed);
    let elapsed_json = serde_json::to_string(&elapsed).unwrap();
    println!("elapsed_json: {}", elapsed_json);

    Ok(())
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
            agg_hash_stream(&filepath).await?;
            // timing_agg_tiles_hash(&filepath)?;
        }
        None => {
            warn!("no fspath provided");
        }
    }
    // DEV END
    Ok(())
}

pub async fn dev_main(args: DevArgs) -> UtilesResult<()> {
    warn!("__DEV_MAIN__");
    dev(args).await?;
    Ok(())
}
