use crate::tile::Tile;

#[derive(Debug)]
pub struct TileData {
    pub xyz: Tile,
    pub data: Vec<u8>,
}
