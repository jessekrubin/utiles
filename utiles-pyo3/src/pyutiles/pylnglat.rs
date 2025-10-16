use crate::pyutiles::pytile::PyTile;
use pyo3::class::basic::CompareOp;
use pyo3::exceptions::{self};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass(name = "LngLat", module = "utiles._utiles", frozen)]
pub struct PyLngLat {
    lnglat: utiles::LngLat,
}

#[pymethods]
impl PyLngLat {
    #[new]
    pub fn py_new(lng: f64, lat: f64) -> Self {
        Self {
            lnglat: utiles::LngLat::new(lng, lat),
        }
    }

    #[classmethod]
    pub fn from_tile(_cls: &Bound<'_, PyType>, tile: &PyTile) -> Self {
        let ll = utiles::ul(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        Self::py_new(ll.lng(), ll.lat())
    }

    pub fn __repr__(&self) -> String {
        format!("LngLat(lng={}, lat={})", self._lng(), self._lat())
    }

    pub fn _lng(&self) -> f64 {
        self.lnglat.lng()
    }

    pub fn _lat(&self) -> f64 {
        self.lnglat.lat()
    }

    #[getter]
    pub fn lng(&self) -> f64 {
        self._lng()
    }

    #[getter]
    pub fn lat(&self) -> f64 {
        self._lat()
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __richcmp__(
        &self,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
    ) -> PyResult<bool> {
        let is_lnglat = other.is_instance_of::<Self>();
        if is_lnglat {
            let maybe_lnglat = other.extract::<PyRef<Self>>();
            if let Ok(lnglat) = maybe_lnglat {
                match op {
                    CompareOp::Eq => {
                        Ok(self._lng() == lnglat._lng() && self._lat() == lnglat._lat())
                    }
                    CompareOp::Ne => {
                        Ok(self._lng() != lnglat._lng() || self._lat() != lnglat._lat())
                    }
                    CompareOp::Lt => {
                        Ok(self._lng() < lnglat._lng() || self._lat() < lnglat._lat())
                    }
                    _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                        "Not implemented",
                    )),
                }
            } else {
                Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                    "Not implemented",
                ))
            }
        } else if let Ok(tuple) = other.extract::<(f64, f64)>() {
            match op {
                CompareOp::Eq => Ok(self._lng() == tuple.0 && self._lat() == tuple.1),
                CompareOp::Ne => Ok(self._lng() != tuple.0 || self._lat() != tuple.1),
                CompareOp::Lt => Ok(self._lng() < tuple.0 || self._lat() < tuple.1),
                _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                    "Not implemented",
                )),
            }
        } else {
            match op {
                CompareOp::Eq | CompareOp::Lt => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                    "Not implemented",
                )),
            }
        }
    }

    pub fn __len__(&self) -> usize {
        2
    }

    pub fn members(&self) -> (f64, f64) {
        self.tuple()
    }

    pub fn __getitem__(&self, idx: i32, _py: Python<'_>) -> PyResult<f64> {
        match idx {
            0 | -2 => Ok(self._lng()),
            1 | -1 => Ok(self._lat()),
            2 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),

            _ => Err(PyErr::new::<exceptions::PyIndexError, _>(
                "Index out of range",
            )),
        }
    }

    pub fn tuple(&self) -> (f64, f64) {
        (self._lng(), self._lat())
    }
}

impl From<utiles::LngLat> for PyLngLat {
    fn from(val: utiles::LngLat) -> Self {
        Self { lnglat: val }
    }
}
