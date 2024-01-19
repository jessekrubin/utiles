use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MbtilesZoomStats {
    pub zoom: u32,
    pub ntiles: i64,
    pub xmin: u32,
    pub xmax: u32,
    pub ymin: u32,
    pub ymax: u32,
}
