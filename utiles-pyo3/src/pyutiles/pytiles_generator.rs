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

    fn __next__(&mut self) -> Option<PyTile> {
        self.iter.next()
    }

    #[expect(clippy::cast_possible_truncation, reason = "TODO")]
    fn __len__(&self) -> usize {
        self.length as usize
    }
}
