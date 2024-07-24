use crate::cli::args::AggHashArgs;
use crate::errors::UtilesResult;
use crate::mbt::hash_types::HashType;
use crate::mbt::mbt_agg_tiles_hash_stream;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

// pub async fn agg_hash_main(args: &AggHashArgs) -> UtilesResult<()> {
//     let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
//     mbt.register_utiles_sqlite_functions().await?;
//     let hash_type = args.hash.unwrap_or(HashType::Md5);
//     let filter = args.filter_args.tiles_filter_maybe();
//     let result = mbt
//         .conn(move |c| Ok(mbt_agg_tiles_hash(c, hash_type, None, &filter)))
//         .await??;
//     println!("{}", serde_json::to_string_pretty(&result)?);
//     Ok(())
// }
pub async fn agg_hash_main(args: &AggHashArgs) -> UtilesResult<()> {
    let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
    mbt.register_utiles_sqlite_functions().await?;
    let hash_type = args.hash.unwrap_or(HashType::Md5);
    let filter = args.filter_args.tiles_filter_maybe();
    let result = mbt_agg_tiles_hash_stream(&mbt, hash_type, None, &filter).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
