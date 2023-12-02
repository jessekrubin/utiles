use crate::{TileLike, flipy};
#[derive(Debug, Clone)]
pub struct MbtTileRow {
    pub zoom_level: u8,
    pub tile_column: u32,
    pub tile_row: u32,
    pub tile_data: Vec<u8>,
}

impl MbtTileRow {
    pub fn new(zoom_level: u8, tile_column: u32, tile_row: u32, tile_data: Vec<u8>) -> Self {
        Self {
            zoom_level,
            tile_column,
            tile_row,
            tile_data,
        }
    }
}


impl TileLike for MbtTileRow {
    fn new(x: u32, y: u32, z: u8) -> Self {
        Self::new(z, x, y, vec![])
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
