/// A struct to hold the minimum and maximum zoom levels for an mbtiles file
#[derive(Debug, Clone, Copy)]
pub struct MinZoomMaxZoom {
    /// minimum zoom level
    pub minzoom: u8,
    /// maximum zoom level
    pub maxzoom: u8,
}

impl MinZoomMaxZoom {
    /// Create a new `MinZoomMaxZoom`
    #[must_use]
    pub fn new(minzoom: u8, maxzoom: u8) -> Self {
        MinZoomMaxZoom { minzoom, maxzoom }
    }
}

impl From<(u8, u8)> for MinZoomMaxZoom {
    fn from(minmax: (u8, u8)) -> Self {
        MinZoomMaxZoom::new(minmax.0, minmax.1)
    }
}
