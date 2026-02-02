use crate::pyutiles::PyLngLatBbox;
use crate::pyutiles::pytile::PyTile;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{PyAny, PyErr, PyRef, PyResult, Python, exceptions, pyclass, pymethods};
use utiles::BBox;

#[pyclass(name = "Bbox", module = "utiles._utiles", frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct PyBbox {
    pub bbox: BBox,
}

#[pymethods]
impl PyBbox {
    #[new]
    pub fn py_new(left: f64, bottom: f64, right: f64, top: f64) -> Self {
        Self {
            bbox: BBox {
                west: left,
                south: bottom,
                east: right,
                north: top,
            },
        }
    }

    #[classmethod]
    pub fn from_tile(_cls: &Bound<'_, PyType>, tile: &PyTile) -> Self {
        let ul = utiles::ul(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        let lr = utiles::lr(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        Self::py_new(ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Bbox(left={}, bottom={}, right={}, top={})",
            self.bbox.left(),
            self.bbox.bottom(),
            self.bbox.right(),
            self.bbox.top()
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    #[getter]
    pub fn left(&self) -> f64 {
        self.bbox.left()
    }

    #[getter]
    pub fn bottom(&self) -> f64 {
        self.bbox.bottom()
    }

    #[getter]
    pub fn right(&self) -> f64 {
        self.bbox.right()
    }

    #[getter]
    pub fn top(&self) -> f64 {
        self.bbox.top()
    }

    #[getter]
    pub fn west(&self) -> f64 {
        self.bbox.left()
    }

    #[getter]
    pub fn south(&self) -> f64 {
        self.bbox.bottom()
    }

    #[getter]
    pub fn east(&self) -> f64 {
        self.bbox.right()
    }

    #[getter]
    pub fn north(&self) -> f64 {
        self.bbox.top()
    }

    pub fn members(&self) -> (f64, f64, f64, f64) {
        self.tuple()
    }

    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        self.bbox.tuple()
    }

    pub fn __len__(&self) -> usize {
        4
    }

    pub fn __getitem__(&self, idx: i32, _py: Python<'_>) -> PyResult<f64> {
        match idx {
            0 | -4 => Ok(self.bbox.left()),
            1 | -3 => Ok(self.bbox.bottom()),
            2 | -2 => Ok(self.bbox.right()),
            3 | -1 => Ok(self.bbox.top()),
            4 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
            _ => Err(PyErr::new::<exceptions::PyIndexError, _>(
                "index out of range (must be -4..4)",
            )),
        }
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
