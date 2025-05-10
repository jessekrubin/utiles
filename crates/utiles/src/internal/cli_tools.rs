//! MISC cli-tools
use tracing::{error, info};

use crate::mbt::{MbtType, Mbtiles};

pub fn open_new_overwrite(
    dst_mbt_path: &str,
    overwrite: bool,
) -> anyhow::Result<Mbtiles> {
    let dst_exists = std::fs::metadata(dst_mbt_path).is_ok();
    if dst_exists {
        if overwrite {
            info!("removing existing mbtiles: {}", dst_mbt_path);
            std::fs::remove_file(dst_mbt_path)?;
        } else {
            error!("dst exists, use --force to overwrite");
            return Err(anyhow::anyhow!("dst exists, use --force to overwrite"));
        }
    }
    let dst = Mbtiles::open_new(dst_mbt_path, Option::from(MbtType::Norm))?;
    Ok(dst)
}
