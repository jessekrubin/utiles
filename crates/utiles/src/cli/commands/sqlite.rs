use tracing::{debug, info, trace};

use crate::cli::args::{AnalyzeArgs, SqliteHeaderArgs};
use crate::errors::UtilesResult;
use crate::fs_async::read_nbytes;
use crate::sqlite::{
    AsyncSqliteConn, Sqlike3Async, SqliteDbAsyncClient, SqliteHeader,
    analysis_limit_set,
};

pub(crate) async fn analyze_main(args: &AnalyzeArgs) -> UtilesResult<()> {
    info!("Analyzing sqlite file: {}", args.common.filepath);
    let db = SqliteDbAsyncClient::open_existing(&args.common.filepath, None).await?;
    if let Some(limit) = args.analysis_limit {
        let limit_i64 = i64::try_from(limit).map_err(|e| {
            crate::errors::UtilesError::AdHoc(format!(
                "analysis_limit value too large (must be positive i64): {e}"
            ))
        })?;
        let current_limit = db.pragma_analysis_limit().await?;
        trace!("current analysis limit: {}", current_limit);
        if current_limit != limit_i64 {
            trace!("setting analysis limit: {} -> {}", current_limit, limit_i64);
            db.conn(move |conn| analysis_limit_set(conn, limit_i64))
                .await?;
        }
    }
    let start_time = std::time::Instant::now();
    db.analyze().await?;
    let analyze_time_ms = start_time.elapsed().as_millis();
    info!("Analyze time: {}ms", analyze_time_ms);
    Ok(())
}

pub(crate) async fn header_main(args: &SqliteHeaderArgs) -> UtilesResult<()> {
    // get the first 100 bytes of the file
    let header_bytes = read_nbytes::<_, 100>(&args.common.filepath).await?;
    debug!("header-bytes: {:?}", header_bytes);
    let header = SqliteHeader::parse(&header_bytes)?;
    header.is_ok()?;
    let json_str = serde_json::to_string_pretty(&header)?;
    println!("{json_str}");
    Ok(())
}
