//! Tile + blob data container
use crate::TileLike;
use crate::tile::Tile;

/// `TileData` container with Tile and u8 bytes
#[derive(Debug, Clone)]
pub struct TileData {
    /// tile x, y, z
    pub xyz: Tile,

    /// tile data
    pub data: Vec<u8>,
}

impl TileData {
    /// Create a new `TileData`
    #[must_use]
    pub fn new(xyz: Tile, data: Vec<u8>) -> Self {
        Self { xyz, data }
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
