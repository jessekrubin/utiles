use std::path::PathBuf;

use tracing::warn;

use crate::cli::args::CopyArgs;
use crate::copy::{copy, CopyConfig};
use crate::errors::UtilesResult;

pub async fn copy_main(args: CopyArgs) -> UtilesResult<()> {
    warn!("experimental command: copy/cp");
    let copy_cfg = CopyConfig {
        src: PathBuf::from(&args.src),
        dst: PathBuf::from(&args.dst),
        zset: args.zoom_set(),
        zooms: args.zooms(),
        verbose: true,
        bboxes: args.bboxes(),
        force: false,
        dryrun: false,
        jobs: args.jobs,
    };
    copy(&copy_cfg).await
}
