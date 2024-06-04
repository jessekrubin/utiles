use tracing::info;

pub use cfg::CopyConfig;
pub use pyramid::copy_mbtiles2fs;
pub use unpyramid::copy_fs2mbtiles;

use crate::errors::UtilesResult;
mod cfg;
mod pasta;
mod pyramid;
mod unpyramid;

pub async fn copy(cfg: &CopyConfig) -> UtilesResult<()> {
    info!("copy-config: {:?}", cfg);

    let cfg_json_str = serde_json::to_string_pretty(cfg)?;
    info!("copy-config-json: {}", cfg_json_str);

    // check and err if necessary
    cfg.check()?;

    Ok(())
}
