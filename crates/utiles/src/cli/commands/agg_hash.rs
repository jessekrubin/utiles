use std::path::Path;

use crate::cli::args::{AggHashArgs, InfoArgs};
use crate::errors::UtilesResult;
use crate::mbt::hash_types::HashType;
use crate::mbt::{mbt_agg_tiles_hash, MbtilesStats, TilesFilter};
use crate::sqlite::AsyncSqliteConn;
use crate::utilesqlite::{Mbtiles, MbtilesAsyncSqliteClient};

pub async fn agg_hash_main(args: &AggHashArgs) -> UtilesResult<()> {
    let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
    let hash_type = args.hash.unwrap_or(HashType::Md5);
    let filter = args.filter_args.tiles_filter_maybe();
    let agg_hash_result = mbt
        .conn(move |c| Ok(mbt_agg_tiles_hash(c, hash_type, None, &filter)))
        .await??;

    // mbt_agg_tiles_hash(
    //
    // //
    // // )
    // let stats = mbinfo(&args.common.filepath)?;
    // let str = if args.common.min {
    //     serde_json::to_string(&stats)
    // } else {
    //     serde_json::to_string_pretty(&stats)
    // }?;
    // println!("{str}");
    Ok(())
}
