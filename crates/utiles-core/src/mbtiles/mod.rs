pub use tiles_row::MbtTileRow;

pub use crate::mbtiles::metadata2map::{metadata2duplicates, metadata2map};
pub use crate::mbtiles::metadata2tilejson::metadata2tilejson;
pub use crate::mbtiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};
pub use crate::mbtiles::minzoom_maxzoom::MinZoomMaxZoom;

mod metadata2map;

mod metadata2tilejson;
pub mod metadata_row;
mod minzoom_maxzoom;
mod tiles_row;

pub const MBTILES_MAGIC_NUMBER: u32 = 0x4d50_4258;
