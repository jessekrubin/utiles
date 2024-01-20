pub use tiles_row::MbtTileRow;

pub use crate::mbutiles::metadata2map::{metadata2duplicates, metadata2map};
pub use crate::mbutiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};
pub use crate::mbutiles::minzoom_maxzoom::MinZoomMaxZoom;

mod metadata2map;

pub mod metadata_row;
mod minzoom_maxzoom;
mod tiles_row;

pub const MBTILES_MAGIC_NUMBER: u32 = 0x4d50_4258;
