use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::cli::args::VacuumArgs;
use crate::errors::UtilesResult;
use crate::utilesqlite::squealite::{Sqlike3, SqliteDb};
use crate::UtilesError;

#[derive(Debug, Serialize, Deserialize)]
pub struct VacuumInfo {
    pub fspath: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<String>,

    pub initial_size: u64,
    pub final_size: u64,
    pub vacuum_time_ms: u128,
    pub analyze_time_ms: u128,
    pub total_time_ms: u128,
    pub size_diff: i64,
}

pub fn vacuum_main(args: &VacuumArgs) -> UtilesResult<()> {
    // check that the file exists
    let db = SqliteDb::open_existing(&args.common.filepath)?;

    // get file size from filepath
    let pre_vac_file_size = std::fs::metadata(&args.common.filepath).unwrap().len();

    // do the vacuum
    let vacuum_start_time = std::time::Instant::now();
    if let Some(dst) = &args.into {
        if dst == &args.common.filepath {
            warn!("Vacuuming into the same file");
        } else {
            // check that the destination file does not exist
            if std::path::Path::new(dst).exists() {
                error!("Destination file already exists: {}", dst);
                return Err(UtilesError::Error(
                    "Destination file already exists".to_string(),
                ));
            }
        }
        info!("vacuuming: {} -> {}", args.common.filepath, dst);
        db.vacuum_into(dst.clone()).unwrap();
    } else {
        info!("vacuuming: {}", args.common.filepath);
        db.vacuum().unwrap();
    }
    let vacuum_time_ms = vacuum_start_time.elapsed().as_millis();

    let analyze_start_time = std::time::Instant::now();
    if let Some(dst) = &args.into {
        let dst_db = SqliteDb::open_existing(dst)?;
        info!("analyzing: {}", dst);
        dst_db.analyze().unwrap();
    } else {
        info!("analyzing: {}", args.common.filepath);
        db.analyze().unwrap();
    }
    let analyze_time_ms = analyze_start_time.elapsed().as_millis();

    // get file size from filepath
    let vacuumed_file_size = match &args.into {
        Some(dst) => std::fs::metadata(dst).unwrap().len(),
        None => std::fs::metadata(&args.common.filepath).unwrap().len(),
    };
    let info = VacuumInfo {
        fspath: args.common.filepath.clone(),
        into: args.into.clone(),
        initial_size: pre_vac_file_size,
        final_size: vacuumed_file_size,
        vacuum_time_ms,
        analyze_time_ms,
        total_time_ms: vacuum_time_ms + analyze_time_ms,
        size_diff: (vacuumed_file_size as i64) - (pre_vac_file_size as i64),
    };
    let out_str = if args.common.min {
        serde_json::to_string(&info).unwrap()
    } else {
        serde_json::to_string_pretty(&info).unwrap()
    };
    println!("{}", out_str);
    Ok(())
}
