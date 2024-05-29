use crate::pyutiles::pytile::PyTile;
use pyo3::FromPyObject;

#[derive(FromPyObject)]
pub struct TileTuple(pub(crate) u32, pub(crate) u32, pub(crate) u8);

impl From<PyTile> for TileTuple {
    fn from(tile: PyTile) -> Self {
        Self(tile.xyz.x, tile.xyz.y, tile.xyz.z)
    }
}

impl From<Vec<u32>> for TileTuple {
    fn from(tile: Vec<u32>) -> Self {
        Self(tile[0], tile[1], tile[2] as u8)
    }
}
