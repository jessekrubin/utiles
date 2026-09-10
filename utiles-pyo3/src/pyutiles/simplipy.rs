use std::collections::HashSet;

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::pyutiles::pyparsing::parse_tiles;
use crate::pyutiles::pytile::PyTile;

#[pyfunction]
#[pyo3(signature = (*args, minzoom = 0))]
pub(crate) fn simplify(
    args: &Bound<'_, PyTuple>,
    minzoom: u8,
) -> PyResult<HashSet<PyTile>> {
    let pytiles_vec = parse_tiles(args)?;
    let pytiles_set = pytiles_vec.into_iter().collect::<HashSet<PyTile>>();
    let root_set = utiles::simplify(&pytiles_set, Some(minzoom));
    Ok(root_set)
}
