use crate::cli::args::CopyArgs;
use crate::copy::{copy, CopyConfig};
use crate::errors::UtilesResult;
use tracing::debug;

pub async fn copy_main(args: CopyArgs) -> UtilesResult<()> {
    debug!("copy-args: {:?}", args);
    let copy_cfg = CopyConfig::from(&args);
    copy(&copy_cfg).await
}
