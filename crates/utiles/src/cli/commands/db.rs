#![allow(dead_code)]
use tracing::debug;

use crate::cli::args::AnalyzeArgs;
use crate::errors::UtilesResult;


pub async fn analyze_main(args: &AnalyzeArgs) -> UtilesResult<()> {
    // check that the file exists
    debug!("analyzing: {}", args.common.filepath);
    Ok(())
}
