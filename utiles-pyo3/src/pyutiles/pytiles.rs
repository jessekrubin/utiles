use crate::pyutiles::pytile::PyTile;

use pyo3::pyclass;
use serde::Serialize;

use std::hash::Hash;

#[pyclass(name = "Tiles", sequence)]
#[derive(Clone, Debug, PartialEq, Serialize, Eq, Hash)]
struct PyTiles {
    tiles: Vec<PyTile>,
}
