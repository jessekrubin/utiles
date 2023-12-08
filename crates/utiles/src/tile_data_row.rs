use crate::tile::Tile;

#[derive(Debug, Clone)]
pub struct TileData {
    pub xyz: Tile,
    pub data: Vec<u8>,
}

impl TileData {
    pub fn new(xyz: Tile, data: Vec<u8>) -> TileData {
        TileData { xyz, data }
    }
}