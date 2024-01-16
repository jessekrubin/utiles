use crate::tile::Tile;
use crate::TileLike;

#[derive(Debug, Clone)]
pub struct TileData {
    pub xyz: Tile,
    pub data: Vec<u8>,
}

impl TileData {
    #[must_use]
    pub fn new(xyz: Tile, data: Vec<u8>) -> TileData {
        TileData { xyz, data }
    }
}

impl TileLike for TileData {
    fn x(&self) -> u32 {
        self.xyz.x
    }

    fn y(&self) -> u32 {
        self.xyz.y
    }

    fn z(&self) -> u8 {
        self.xyz.z
    }
}
