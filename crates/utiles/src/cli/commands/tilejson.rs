use tracing::debug;

use crate::cli::args::TilejsonArgs;
use crate::errors::UtilesResult;
use crate::utilejson::tilejson_stringify;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

pub async fn tilejson_main(args: &TilejsonArgs) -> UtilesResult<()> {
    debug!("tilejson: {}", args.common.filepath);
    let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
    let mut tj = mbt.tilejson().await?;
    if !args.tilestats {
        tj.other.remove("tilestats");
    }
    let s = tilejson_stringify(&tj, Option::from(!args.common.min));
    println!("{s}");
    Ok(())
}

// use tracing::debug;
//
// use crate::cli::args::TilejsonArgs;
// use crate::errors::UtilesResult;
// use crate::pmt::fspath2pmtilejson;
// use crate::utilejson::tilejson_stringify;
// use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
//
// pub async fn tilejson_main(args: &TilejsonArgs) -> UtilesResult<()> {
//     debug!("tilejson: {}", args.common.filepath);
//     // if it ends with .pmtiles, use pmtiles else use mbtiles...
//     let mut tj = if args.common.filepath.ends_with(".pmtiles") {
//         // pmtiles
//         fspath2pmtilejson(&args.common.filepath).await?
//         // let reader = AsyncPmTilesReader::new_with_path(&args.common.filepath).await?;
//         // reader.parse_tilejson(vec![]).await?
//     } else {
//         // mbtiles
//         let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
//         let tj = mbt.tilejson().await?;
//         tj
//     };
//     // let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
//     // let mut tj = mbt.tilejson().await?;
//     if !args.tilestats {
//         tj.other.remove("tilestats");
//     }
//     let s = tilejson_stringify(&tj, Option::from(!args.common.min));
//     println!("{s}");
//     Ok(())
// }
