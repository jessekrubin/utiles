use pyo3::{pyclass, pymethods, PyRef, PyRefMut};

#[pyclass]
pub struct IntIterator {
    pub iter: Box<dyn Iterator<Item = u32> + Send>,
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
pub struct FloatIterator {
    pub iter: Box<dyn Iterator<Item = f64> + Send>,
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

#[pymethods]
impl CoordinateIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(f64, f64)> {
        slf.iter.next()
    }
}

#[pyclass]
pub struct CoordinateIterator {
    pub iter: Box<dyn Iterator<Item = (f64, f64)> + Send>,
}
