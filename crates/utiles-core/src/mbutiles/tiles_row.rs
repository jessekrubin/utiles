use crate::tile_type::tiletype_str;
use crate::{flipy, TileLike};
#[derive(Debug, Clone)]
pub struct MbtTileRow {
    pub zoom_level: u8,
    pub tile_column: u32,
    pub tile_row: u32,
    pub tile_data: Vec<u8>,
}

impl MbtTileRow {
    #[must_use]
    pub fn new(
        zoom_level: u8,
        tile_column: u32,
        tile_row: u32,
        tile_data: Vec<u8>,
    ) -> Self {
        Self {
            zoom_level,
            tile_column,
            tile_row,
            tile_data,
        }
    }
}

impl TileLike for MbtTileRow {
    fn x(&self) -> u32 {
        self.tile_column
    }

    fn y(&self) -> u32 {
        flipy(self.tile_row, self.zoom_level)
    }

    fn z(&self) -> u8 {
        self.zoom_level
    }
}

impl MbtTileRow {
    #[must_use]
    pub fn extension(&self) -> String {
        tiletype_str(&self.tile_data)
    }
}
