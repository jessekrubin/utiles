use std::fs::canonicalize;
use std::path::Path;

use crate::cli::args::InfoArgs;
use crate::errors::UtilesResult;
use crate::fs_async::file_exists;
use crate::mbt::MbtilesStats;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

async fn mbinfo(filepath: &str) -> UtilesResult<MbtilesStats> {
    let fspath = Path::new(filepath);
    let is_file = file_exists(filepath).await;
    if !is_file {
        let abspath = canonicalize(fspath)?;

        Err(crate::errors::UtilesError::NotAFile(
            abspath.to_string_lossy().to_string(),
        ))
    } else {
        let mbt = MbtilesAsyncSqliteClient::open_existing(filepath).await?;
        let stats = mbt.mbt_stats(None).await?;
        Ok(stats)
    }
}

pub async fn info_main(args: &InfoArgs) -> UtilesResult<()> {
    let stats = mbinfo(&args.common.filepath).await?;
    let str = if args.common.min {
        serde_json::to_string(&stats)
    } else {
        serde_json::to_string_pretty(&stats)
    }?;
    println!("{str}");
    Ok(())
}
