use pyo3::prelude::*;

use crate::pyutiles::pytile::PyTile;

#[pyclass(name = "TilesGenerator", module = "utiles._utiles")]
pub struct TilesGenerator {
    pub(crate) iter: Box<dyn Iterator<Item = PyTile> + Send + Sync>,
    pub(crate) length: u64,
}

#[pymethods]
impl TilesGenerator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTile> {
        slf.iter.next()
    }

    fn __len__(slf: PyRefMut<'_, Self>) -> usize {
        slf.length as usize
    }
}
