use std::collections::HashSet;

use pyo3::types::PyTuple;
use pyo3::{pyfunction, Bound, PyResult};

use crate::pyutiles::pyparsing::parse_tiles;
use crate::pyutiles::pytile::PyTile;

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn simplify(args: &Bound<'_, PyTuple>) -> PyResult<HashSet<PyTile>> {
    let pytiles_vec = parse_tiles(args)?;
    let pytiles_set = pytiles_vec.into_iter().collect::<HashSet<PyTile>>();
    let root_set = utiles::simplify(&pytiles_set);
    Ok(root_set)
}
