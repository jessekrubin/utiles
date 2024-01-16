use crate::fns::flipy;
use crate::tile_like::TileLike;
use crate::Tile;

#[derive(Debug)]
struct TileCrz {
    // column -> x
    tile_column: u32,
    // row -> y
    tile_row: u32,
    // zoom_level -> z
    zoom_level: u8,
}

impl TileCrz {
    pub fn new(tile_column: u32, tile_row: u32, zoom_level: u8) -> Self {
        Self {
            tile_column,
            tile_row,
            zoom_level,
        }
    }
}

impl From<Tile> for TileCrz {
    fn from(tile: Tile) -> Self {
        Self::new(tile.x, flipy(tile.y, tile.z), tile.z)
    }
}

impl From<TileCrz> for Tile {
    fn from(tile: TileCrz) -> Self {
        Self::new(
            tile.tile_column,
            flipy(tile.tile_row, tile.zoom_level),
            tile.zoom_level,
        )
    }
}

impl TileLike for TileCrz {
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
