use std::collections::HashSet;

use crate::pyutiles::pyparsing::parse_tiles;
use crate::pyutiles::pytile::PyTile;
use pyo3::types::PyTuple;
use pyo3::{pyfunction, Bound, PyResult};

#[pyfunction]
#[pyo3(signature = (* args, minzoom=None))]
pub fn simplify(
    args: &Bound<'_, PyTuple>,
    minzoom: Option<u8>,
) -> PyResult<HashSet<PyTile>> {
    let pytiles_vec = parse_tiles(args)?;
    let pytiles_set = pytiles_vec.into_iter().collect::<HashSet<PyTile>>();
    let root_set = utiles::simplify(&pytiles_set, minzoom);
    Ok(root_set)
}
