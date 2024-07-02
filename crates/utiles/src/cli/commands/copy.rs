use crate::cli::args::CopyArgs;
use crate::copy::{copy, CopyConfig};
use crate::errors::UtilesResult;

#[tracing::instrument]
pub async fn copy_main(args: CopyArgs) -> UtilesResult<()> {
    let copy_cfg = CopyConfig::from(&args);
    copy(&copy_cfg).await
}
