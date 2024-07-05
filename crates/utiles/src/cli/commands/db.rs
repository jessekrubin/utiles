#![allow(dead_code)]
use tracing::debug;

use crate::cli::args::AnalyzeArgs;
use crate::errors::UtilesResult;
use crate::utilesqlite::MbtilesAsyncSqliteClient;

pub async fn analyze_main(args: &AnalyzeArgs) -> UtilesResult<()> {
    // check that the file exists
    debug!("analyzing: {}", args.common.filepath);
    let mbt = MbtilesAsyncSqliteClient::open_existing(&args.common.filepath).await?;
    debug!("mbt: {:?}", mbt);
    Ok(())
}
