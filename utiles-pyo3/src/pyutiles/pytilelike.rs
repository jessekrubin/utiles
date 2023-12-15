use crate::pyutiles::pytile::PyTile;
use pyo3::FromPyObject;

#[derive(FromPyObject)]
pub enum PyTileLike {
    #[pyo3(transparent, annotation = "tuple[int, int, int]")]
    Tuple3d((u32, u32, u8)),

    #[pyo3(transparent, annotation = "Tile")]
    PyTile(PyTile),
}

impl From<PyTileLike> for PyTile {
    fn from(val: PyTileLike) -> Self {
        match val {
            PyTileLike::Tuple3d((x, y, z)) => PyTile::from(utiles::Tile::new(x, y, z)),
            PyTileLike::PyTile(t) => t,
        }
    }
}
