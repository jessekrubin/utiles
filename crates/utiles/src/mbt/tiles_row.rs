use utiles_core::Tile;

use crate::core::tile_type::tiletype_str;
use crate::core::{flipy, TileLike};

/// Mbtiles Tile Row struct
#[derive(Debug, Clone)]
pub struct MbtTileRow {
    /// `zoom_level` INTEGER NOT NULL -- z
    pub zoom_level: u8,
    /// `tile_column` INTEGER NOT NULL -- x
    pub tile_column: u32,
    /// `tile_row` INTEGER NOT NULL -- y (flipped)
    pub tile_row: u32,
    /// `tile_data` BLOB NOT NULL -- tile data
    pub tile_data: Vec<u8>,
}

impl MbtTileRow {
    /// Create a new `MbtTileRow`
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
    /// Return the file extension of the tile based on the `tile_data`
    #[must_use]
    pub fn extension(&self) -> String {
        tiletype_str(&self.tile_data)
    }
}

impl From<MbtTileRow> for Tile {
    fn from(row: MbtTileRow) -> Self {
        row.tile()
    }
}
