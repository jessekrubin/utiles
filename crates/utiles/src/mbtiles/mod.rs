mod metadata2map;
mod metadata2tilejson;
pub mod metadata_row;
mod minzoom_maxzoom;

pub use crate::mbtiles::metadata2map::{metadata2duplicates, metadata2map};
pub use crate::mbtiles::metadata2tilejson::metadata2tilejson;
pub use crate::mbtiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};
pub use crate::mbtiles::minzoom_maxzoom::MinZoomMaxZoom;

pub const MBTILES_MAGIC_NUMBER: u32 = 0x4d504258;
