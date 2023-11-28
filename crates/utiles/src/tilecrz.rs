use crate::fns::flipy;
use crate::tile_like::TileLike;

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

impl TileLike for TileCrz {
    fn new(x: u32, y: u32, z: u8) -> Self {
        Self::new(x, flipy(y, z), z)
    }

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
