#![allow(dead_code)]
use tracing::{info, trace};

use crate::cli::args::AnalyzeArgs;
use crate::errors::UtilesResult;
use crate::sqlite::{
    analysis_limit_set, AsyncSqliteConn, Sqlike3Async, SqliteDbAsyncClient,
};

pub async fn analyze_main(args: &AnalyzeArgs) -> UtilesResult<()> {
    info!("Analyzing sqlite file: {}", args.common.filepath);
    let db = SqliteDbAsyncClient::open_existing(&args.common.filepath, None).await?;
    if args.analysis_limit.is_some() {
        let current_limit = db.pragma_analysis_limit().await?;
        if let Some(limit) = args.analysis_limit {
            if current_limit != limit {
                trace!("setting analysis limit: {} -> {}", current_limit, limit);
                db.conn(move |conn| analysis_limit_set(conn, limit)).await?;
            }
        }
    }
    let start_time = std::time::Instant::now();
    db.analyze().await?;
    let analyze_time_ms = start_time.elapsed().as_millis();
    info!("Analyze time: {}ms", analyze_time_ms);
    Ok(())
}

//
// pub async fn schema_main(args: &SqliteSchemaArgs) -> UtilesResult<()> {
//     info!("Schema for sqlite file: {}", args.common.filepath);
//     let db = SqliteDbAsyncClient::open_existing(&args.common.filepath, None).await?;
//     let schema = db.schema().await?;
//     println!("{}", schema);
//     Ok(())
// }
