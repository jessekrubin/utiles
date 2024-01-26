use crate::cli::args::SqliteDbCommonArgs;
use crate::utilesqlite::squealite::Sqlike3;
use crate::utilesqlite::Mbtiles;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VacuumInfo {
    pub fspath: String,
    pub initial_size: u64,
    pub final_size: u64,
    pub vacuum_time_ms: u128,
    pub analyze_time_ms: u128,
    pub total_time_ms: u128,
    pub size_diff: i64,
}

pub fn vacuum_main(args: &SqliteDbCommonArgs) {
    // check that the file exists
    let mbt = Mbtiles::open_existing(&args.filepath).unwrap();
    // get file size from filepath
    let pre_vac_file_size = std::fs::metadata(&args.filepath).unwrap().len();
    let vacuum_start_time = std::time::Instant::now();
    mbt.vacuum().unwrap();
    let vacuum_time_ms = vacuum_start_time.elapsed().as_millis();
    let analyze_start_time = std::time::Instant::now();
    mbt.analyze().unwrap();
    let analyze_time_ms = analyze_start_time.elapsed().as_millis();
    let vacuumed_file_size = std::fs::metadata(&args.filepath).unwrap().len();

    let info = VacuumInfo {
        fspath: args.filepath.clone(),
        initial_size: pre_vac_file_size,
        final_size: vacuumed_file_size,
        vacuum_time_ms,
        analyze_time_ms,
        total_time_ms: vacuum_time_ms + analyze_time_ms,
        size_diff: (pre_vac_file_size as i64) - (vacuumed_file_size as i64),
    };
    let out_str = if args.min {
        serde_json::to_string(&info).unwrap()
    } else {
        serde_json::to_string_pretty(&info).unwrap()
    };
    println!("{}", out_str);
}
