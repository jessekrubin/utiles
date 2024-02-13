use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::cli::args::VacuumArgs;
use crate::utilesqlite::squealite::Sqlike3;
use crate::utilesqlite::Mbtiles;

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

pub fn vacuum_main(args: &VacuumArgs) {
    // check that the file exists
    let mbt = Mbtiles::open_existing(&args.common.filepath).unwrap();
    // get file size from filepath
    let pre_vac_file_size = std::fs::metadata(&args.common.filepath).unwrap().len();

    // do the vacuum
    let vacuum_start_time = std::time::Instant::now();
    if let Some(dst) = &args.into {
        if dst == &args.common.filepath {
            error!("Cannot vacuum into the same file");
            panic!("Cannot vacuum into the same file");
        }
        // check that the destination file does not exist
        if std::path::Path::new(dst).exists() {
            error!("Destination file already exists: {}", dst);
            panic!("Destination file already exists: {}", dst);
        }
        info!("vacuuming: {} -> {}", args.common.filepath, dst);
        mbt.vacuum_into(dst.clone()).unwrap();
    } else {
        info!("vacuuming: {}", args.common.filepath);
        mbt.vacuum().unwrap();
    }
    let vacuum_time_ms = vacuum_start_time.elapsed().as_millis();

    let analyze_start_time = std::time::Instant::now();
    if let Some(dst) = &args.into {
        let mbt_dst = Mbtiles::open_existing(dst).unwrap();
        info!("analyzing: {}", dst);
        mbt_dst.analyze().unwrap();
    } else {
        info!("analyzing: {}", args.common.filepath);
        mbt.analyze().unwrap();
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
}
