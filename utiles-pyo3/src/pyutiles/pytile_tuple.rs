use pyo3::prelude::*;

use crate::pyutiles::pytile::PyTile;

#[derive(FromPyObject)]
pub struct TileTuple(pub(crate) u32, pub(crate) u32, pub(crate) u8);

impl From<PyTile> for TileTuple {
    fn from(tile: PyTile) -> Self {
        Self(tile.xyz.x, tile.xyz.y, tile.xyz.z)
    }
}
