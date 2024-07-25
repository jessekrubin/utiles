//! Optimize command (optimizes for size)
//!
//! Will optimize db containing tiles (eg mbtiles) but not implemented yet!
//!
//! Plan on using oxipng for pngs and checking if de-duping tiles is worth it.
use tracing::{info, warn};

use crate::cli::args::OptimizeArgs;
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::UtilesResult;

pub async fn optimize_main(args: OptimizeArgs) -> UtilesResult<()> {
    info!("Optimizing mbtiles file: {}", args.common.filepath);
    warn!("NOT IMPLEMENTED YET");
    let mbt = MbtilesClientAsync::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;
    Ok(())
}
