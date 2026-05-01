use pyo3::prelude::*;

#[pyclass]
pub(crate) struct IntIterator {
    pub(crate) iter: Box<dyn Iterator<Item = u32> + Send + Sync>,
}

#[pymethods]
impl IntIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u32> {
        slf.iter.next()
    }
}

#[pyclass]
pub(crate) struct FloatIterator {
    pub(crate) iter: Box<dyn Iterator<Item = f64> + Send + Sync>,
}

#[pymethods]
impl FloatIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<f64> {
        slf.iter.next()
    }
}

#[pyclass]
pub(crate) struct CoordinateIterator {
    pub(crate) iter: Box<dyn Iterator<Item = (f64, f64)> + Send + Sync>,
}

#[pymethods]
impl CoordinateIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(f64, f64)> {
        slf.iter.next()
    }
}
