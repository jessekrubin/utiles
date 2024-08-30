use std::fs::canonicalize;
use std::path::Path;

use crate::errors::UtilesResult;
use crate::fs_async::file_exists;
use crate::mbt::MbtilesStats;
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};

pub async fn mbinfo(filepath: &str, stats: Option<bool>) -> UtilesResult<MbtilesStats> {
    let fspath = Path::new(filepath);
    let is_file = file_exists(filepath).await;
    if is_file {
        let mbt = MbtilesClientAsync::open_existing(filepath).await?;
        let stats = mbt.mbt_stats(stats).await?;
        Ok(stats)
    } else {
        let abspath = canonicalize(fspath)?;

        Err(crate::errors::UtilesError::NotAFile(
            abspath.to_string_lossy().to_string(),
        ))
    }
}
