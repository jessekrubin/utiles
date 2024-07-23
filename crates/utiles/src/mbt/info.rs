use std::fs::canonicalize;
use std::path::Path;

use crate::errors::UtilesResult;
use crate::fs_async::file_exists;
use crate::mbt::MbtilesStats;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

pub async fn mbinfo(filepath: &str) -> UtilesResult<MbtilesStats> {
    let fspath = Path::new(filepath);
    let is_file = file_exists(filepath).await;
    if is_file {
        let mbt = MbtilesAsyncSqliteClient::open_existing(filepath).await?;
        let stats = mbt.mbt_stats(None).await?;
        Ok(stats)
    } else {
        let abspath = canonicalize(fspath)?;

        Err(crate::errors::UtilesError::NotAFile(
            abspath.to_string_lossy().to_string(),
        ))
    }
}
