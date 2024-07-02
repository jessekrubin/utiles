use crate::mbt::MbtType;
use serde::Serialize;

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct MbtilesStats {
    pub filesize: u64,
    pub mbtype: MbtType,
    pub ntiles: u64,
    pub nzooms: u32,
    pub page_count: i64,
    pub page_size: i64,
    pub freelist_count: i64,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub zooms: Vec<MbtilesZoomStats>,
}
