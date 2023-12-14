use std::path::Path;

use serde::Serialize;
use tracing::debug;

use utilesqlite::mbtiles::MbtilesZoomStats;
use utilesqlite::Mbtiles;

use crate::args::MbtilesStatsArgs;

#[derive(Debug, Serialize)]
struct MbtilesStats {
    filesize: u64,
    ntiles: u64,
    nzooms: u32,
    minzoom: Option<u8>,
    maxzoom: Option<u8>,
    zooms: Vec<MbtilesZoomStats>,
}

fn mbinfo(filepath: &str) -> Result<MbtilesStats, Box<dyn std::error::Error>> {
    let filepath = Path::new(filepath);
    assert!(
        filepath.exists(),
        "File does not exist: {}",
        filepath.display()
    );
    assert!(
        filepath.is_file(),
        "Not a file: {filepath}",
        filepath = filepath.display()
    );

    let filesize = filepath.metadata().unwrap().len();
    let mbtiles: Mbtiles = Mbtiles::from(filepath);

    let query_ti = std::time::Instant::now();
    debug!("Started zoom_stats query");
    let zoom_stats = mbtiles.zoom_stats().expect("zoom_stats query failed");
    let query_dt = query_ti.elapsed();
    debug!("Finished zoom_stats query in {:?}", query_dt);

    if zoom_stats.is_empty() {
        return Ok(MbtilesStats {
            filesize,
            ntiles: 0,
            minzoom: None,
            maxzoom: None,
            nzooms: 0,
            zooms: vec![],
        });
    }

    let minzoom = zoom_stats.iter().map(|r| r.zoom).min();
    let maxzoom = zoom_stats.iter().map(|r| r.zoom).max();
    let minzoom_u8: Option<u8> = minzoom.map(|minzoom| minzoom.try_into().unwrap());
    let maxzoom_u8: Option<u8> = maxzoom.map(|maxzoom| maxzoom.try_into().unwrap());

    Ok(MbtilesStats {
        ntiles: zoom_stats.iter().map(|r| r.ntiles as u64).sum(),
        filesize,
        minzoom: minzoom_u8,
        maxzoom: maxzoom_u8,
        nzooms: zoom_stats.len() as u32,
        zooms: zoom_stats,
    })
}

pub fn mbtiles_info_main(args: &MbtilesStatsArgs) {
    let stats = mbinfo(&args.common.filepath);
    match stats {
        Ok(stats) => {
            let str = match args.common.min {
                true => serde_json::to_string(&stats).unwrap(),
                false => serde_json::to_string_pretty(&stats).unwrap(),
            };
            println!("{str}");
        }
        Err(e) => {
            println!("Error: {e}");
        }
    }
}
