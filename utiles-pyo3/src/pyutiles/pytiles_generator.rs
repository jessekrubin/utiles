use pyo3::{pyclass, pymethods, PyRef, PyRefMut};

use crate::pyutiles::pytile::PyTile;

#[pyclass]
pub struct TilesGenerator {
    pub iter: Box<dyn Iterator<Item = PyTile> + Send>,
    pub length: u64,
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
