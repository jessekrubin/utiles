use crate::pyutiles::pyiters::FloatIterator;
use crate::pyutiles::pytile::PyTile;

use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{exceptions, pyclass, pymethods, Py, PyAny, PyErr, PyRef, PyResult, Python};
use utiles::bbox::BBox;

#[pyclass(name = "LngLatBbox", module = "utiles._utiles")]
#[derive(Clone)]
pub struct PyLngLatBbox {
    pub bbox: BBox,
}

impl From<PyLngLatBbox> for BBox {
    fn from(val: PyLngLatBbox) -> Self {
        val.bbox
    }
}

#[pymethods]
impl PyLngLatBbox {
    #[new]
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        PyLngLatBbox {
            bbox: BBox {
                west,
                south,
                east,
                north,
            },
        }
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<FloatIterator>> {
        let iter = FloatIterator {
            iter: Box::new(
                vec![
                    slf.bbox.west(),
                    slf.bbox.south(),
                    slf.bbox.east(),
                    slf.bbox.north(),
                ]
                .into_iter(),
            ),
        };
        Py::new(slf.py(), iter)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "LngLatBbox(west={}, south={}, east={}, north={})",
            self.bbox.west(),
            self.bbox.south(),
            self.bbox.east(),
            self.bbox.north()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    #[classmethod]
    pub fn from_tile(_cls: &Bound<'_, PyType>, tile: &PyTile) -> Self {
        let ul = utiles::ul(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        let lr = utiles::lr(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        Self::new(ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    #[getter]
    pub fn west(&self) -> f64 {
        self.bbox.west()
    }

    #[getter]
    pub fn south(&self) -> f64 {
        self.bbox.south()
    }

    #[getter]
    pub fn east(&self) -> f64 {
        self.bbox.east()
    }

    #[getter]
    pub fn north(&self) -> f64 {
        self.bbox.north()
    }

    pub fn members(&self) -> (f64, f64, f64, f64) {
        self.tuple()
    }

    pub fn __len__(&self) -> usize {
        4
    }

    pub fn __getitem__(&self, idx: i32, _py: Python<'_>) -> PyResult<f64> {
        match idx {
            0 | -4 => Ok(self.bbox.west),
            1 | -3 => Ok(self.bbox.south),
            2 | -2 => Ok(self.bbox.east),
            3 | -1 => Ok(self.bbox.north),
            4 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
            _ => Err(PyErr::new::<exceptions::PyIndexError, _>(
                "Index out of range for BBox",
            )),
        }
    }

    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        self.bbox.tuple()
    }

    pub fn __richcmp__(
        &self,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
    ) -> PyResult<bool> {
        let maybetuple = other.extract::<(f64, f64, f64, f64)>();

        if let Ok(tuple) = maybetuple {
            match op {
                CompareOp::Eq => Ok(self.bbox.west() == tuple.0
                    && self.bbox.south() == tuple.1
                    && self.bbox.east() == tuple.2
                    && self.bbox.north() == tuple.3),
                CompareOp::Ne => Ok(self.bbox.west() != tuple.0
                    || self.bbox.south() != tuple.1
                    || self.bbox.east() != tuple.2
                    || self.bbox.north() != tuple.3),
                CompareOp::Lt => Ok(self.bbox.west() < tuple.0
                    || self.bbox.south() < tuple.1
                    || self.bbox.east() < tuple.2
                    || self.bbox.north() < tuple.3),
                _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                    "Not implemented",
                )),
            }
        } else {
            let other = other.extract::<PyRef<PyLngLatBbox>>();
            match other {
                Ok(other) => match op {
                    CompareOp::Eq => Ok(self.bbox.west() == other.bbox.west()
                        && self.bbox.south() == other.bbox.south()
                        && self.bbox.east() == other.bbox.east()
                        && self.bbox.north() == other.bbox.north()),
                    CompareOp::Ne => Ok(self.bbox.west != other.bbox.west()
                        || self.bbox.south() != other.bbox.south()
                        || self.bbox.east() != other.bbox.east()
                        || self.bbox.north() != other.bbox.north()),
                    CompareOp::Lt => Ok(self.bbox.west() < other.bbox.west()
                        || self.bbox.south() < other.bbox.south()
                        || self.bbox.east() < other.bbox.east()
                        || self.bbox.north() < other.bbox.north()),
                    _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                        "Not implemented",
                    )),
                },
                Err(_) => match op {
                    CompareOp::Eq => Ok(false),
                    CompareOp::Ne => Ok(true),
                    _ => Err(PyErr::new::<exceptions::PyNotImplementedError, _>(
                        "Not implemented",
                    )),
                },
            }
        }
    }
}
