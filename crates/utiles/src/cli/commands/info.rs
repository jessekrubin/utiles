use std::path::Path;

use serde::Serialize;
use tracing::debug;

use crate::cli::args::InfoArgs;
use crate::errors::UtilesResult;
use crate::sqlite::Sqlike3;
use crate::utilesqlite::mbtstats::MbtilesZoomStats;
use crate::utilesqlite::Mbtiles;

#[derive(Debug, Serialize)]
struct MbtilesStats {
    filesize: u64,
    ntiles: u64,
    nzooms: u32,
    page_count: i64,
    page_size: i64,
    minzoom: Option<u8>,
    maxzoom: Option<u8>,
    zooms: Vec<MbtilesZoomStats>,
}

fn mbinfo(filepath: &str) -> UtilesResult<MbtilesStats> {
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

    let page_count = mbtiles.pragma_page_count()?;

    let page_size = mbtiles.pragma_page_size()?;

    let query_ti = std::time::Instant::now();
    debug!("Started zoom_stats query");
    let zoom_stats = mbtiles.zoom_stats().expect("zoom_stats query failed");
    let query_dt = query_ti.elapsed();
    debug!("Finished zoom_stats query in {:?}", query_dt);

    if zoom_stats.is_empty() {
        return Ok(MbtilesStats {
            filesize,
            page_count,
            page_size,
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
        page_count,
        page_size,
        minzoom: minzoom_u8,
        maxzoom: maxzoom_u8,
        nzooms: zoom_stats.len() as u32,
        zooms: zoom_stats,
    })
}

pub fn info_main(args: &InfoArgs) -> UtilesResult<()> {
    let stats = mbinfo(&args.common.filepath)?;
    let str = if args.common.min {
        serde_json::to_string(&stats).unwrap()
    } else {
        serde_json::to_string_pretty(&stats).unwrap()
    };
    println!("{str}");
    Ok(())
}
