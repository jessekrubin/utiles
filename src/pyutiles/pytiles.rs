use crate::pyutiles::pytile::PyTile;

use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use std::hash::Hash;

#[pyclass(name = "Tile", sequence)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
struct PyTiles {
    tiles: Vec<PyTile>,
}
