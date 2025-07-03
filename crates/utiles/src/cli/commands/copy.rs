use crate::cli::args::CopyArgs;
use crate::copy::{CopyConfig, copy};
use crate::errors::UtilesResult;
use tracing::debug;

pub(crate) async fn copy_main(args: CopyArgs) -> UtilesResult<()> {
    debug!("copy-args: {:?}", args);
    let copy_cfg = CopyConfig::from(&args);
    copy(&copy_cfg).await
}
