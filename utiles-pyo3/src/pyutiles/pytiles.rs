use std::hash::Hash;

use pyo3::pyclass;
use serde::Serialize;

use crate::pyutiles::pytile::PyTile;

#[pyclass(
    name = "Tiles",
    module = "utiles._utiles",
    sequence,
    frozen,
    skip_from_py_object
)]
#[derive(Clone, Debug, PartialEq, Serialize, Eq, Hash)]
struct PyTiles {
    tiles: Vec<PyTile>,
}
