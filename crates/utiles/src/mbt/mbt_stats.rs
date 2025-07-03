use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::mbt::mbtiles::{zoom_stats, zoom_stats_full};
use crate::mbt::query::query_mbtiles_type;
use crate::sqlite::{pragma_freelist_count, pragma_page_count, pragma_page_size};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesZoomStats {
    pub zoom: u32,
    pub ntiles: u64,
    pub xmin: u32,
    pub xmax: u32,
    pub ymin: u32,
    pub ymax: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbytes_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesStats {
    pub filesize: u64,
    pub mbtype: MbtType,
    pub ntiles: u64,
    pub nzooms: u32,
    pub page_count: i64,
    pub page_size: i64,
    pub freelist_count: i64,
    pub minzoom: Option<u32>,
    pub maxzoom: Option<u32>,
    pub zooms: Vec<MbtilesZoomStats>,
}

pub fn query_mbt_stats(
    conn: &Connection,
    full: Option<bool>,
) -> UtilesResult<MbtilesStats> {
    let query_ti = std::time::Instant::now();
    let maybe_filepath = conn.path().map(|p| p.to_string());
    let filesize = match maybe_filepath {
        Some(fp) => std::fs::metadata(fp).map(|md| md.len()).unwrap_or(0),
        None => 0,
    };

    // let zoom_stats_full = full.unwrap_or(false) || filesize < 10_000_000_000;
    debug!("Started zoom_stats query");
    let page_count = pragma_page_count(conn)?;

    let page_size = pragma_page_size(conn, None)?;
    let freelist_count = pragma_freelist_count(conn)?;
    // if the file is over 10gb and full is None or false just don't do the
    // zoom_stats query that counts size... bc it is slow af
    // let zoom_stats = self.zoom_stats(zoom_stats_full)?;
    let zoom_stats =
        if full.unwrap_or(false) || (filesize < 10_000_000_000 && filesize > 0) {
            zoom_stats_full(conn)?
        } else {
            zoom_stats(conn)?
        };
    debug!("zoom_stats: {:?}", zoom_stats);
    let query_dt = query_ti.elapsed();
    debug!("Finished zoom_stats query in {:?}", query_dt);
    let mbt_type = query_mbtiles_type(conn)?;
    if zoom_stats.is_empty() {
        return Ok(MbtilesStats {
            filesize,
            mbtype: mbt_type,
            page_count,
            page_size,
            freelist_count,
            ntiles: 0,
            minzoom: None,
            maxzoom: None,
            nzooms: 0,
            zooms: vec![],
        });
    }

    let minzoom = zoom_stats.iter().map(|r| r.zoom).min();
    let maxzoom = zoom_stats.iter().map(|r| r.zoom).max();
    Ok(MbtilesStats {
        ntiles: zoom_stats.iter().map(|r| r.ntiles).sum(),
        filesize,
        mbtype: mbt_type,
        page_count,
        page_size,
        freelist_count,
        minzoom,
        maxzoom,
        nzooms: zoom_stats.len() as u32,
        zooms: zoom_stats,
    })
}
