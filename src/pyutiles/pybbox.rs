use crate::pyutiles::pytile::PyTile;
use crate::utiles;
use crate::utiles::BBox;
use pyo3::basic::CompareOp;
use pyo3::types::PyType;
use pyo3::{
    exceptions, pyclass, pymethods, IntoPy, PyAny, PyErr, PyObject, PyRef, PyResult,
    Python,
};

#[pyclass(name = "Bbox")]
#[derive(Clone)]
pub struct PyBbox {
    pub bbox: BBox,
}

#[pymethods]
impl PyBbox {
    #[new]
    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> Self {
        PyBbox {
            bbox: BBox {
                west: left,
                south: bottom,
                east: right,
                north: top,
            },
        }
    }

    #[classmethod]
    pub fn from_tile(_cls: &PyType, tile: &PyTile) -> PyResult<Self> {
        let ul = utiles::ul(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        let lr = utiles::lr(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        Ok(Self::new(ul.lng(), lr.lat(), lr.lng(), ul.lat()))
    }

    pub fn __str__(&self) -> String {
        format!(
            "Bbox(left={}, bottom={}, right={}, top={})",
            self.bbox.left(),
            self.bbox.bottom(),
            self.bbox.right(),
            self.bbox.top()
        )
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    #[getter]
    pub fn left(&self) -> PyResult<f64> {
        Ok(self.bbox.left())
    }

    #[getter]
    pub fn bottom(&self) -> PyResult<f64> {
        Ok(self.bbox.bottom())
    }

    #[getter]
    pub fn right(&self) -> PyResult<f64> {
        Ok(self.bbox.right())
    }

    #[getter]
    pub fn top(&self) -> PyResult<f64> {
        Ok(self.bbox.top())
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
            0 => Ok(self.bbox.left()),
            1 => Ok(self.bbox.bottom()),
            2 => Ok(self.bbox.right()),
            3 => Ok(self.bbox.top()),
            -1 => Ok(self.bbox.top()),
            -2 => Ok(self.bbox.right()),
            -3 => Ok(self.bbox.bottom()),
            -4 => Ok(self.bbox.left()),
            4 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
            _ => Err(PyErr::new::<exceptions::PyIndexError, _>(
                "index out of range (must be -4..4)",
            )),
        }
    }

    pub fn __richcmp__(
        &self,
        other: &PyAny,
        op: CompareOp,
        py: Python<'_>,
    ) -> PyObject {
        // fn __richcmp__(&self, other: PyAny, op: CompareOp, py: Python<'_>) -> PyObject {
        let maybetuple = other.extract::<(f64, f64, f64, f64)>();

        if let Ok(tuple) = maybetuple {
            match op {
                CompareOp::Eq => (self.bbox.left() == tuple.0
                    && self.bbox.bottom() == tuple.1
                    && self.bbox.right() == tuple.2
                    && self.bbox.top() == tuple.3)
                    .into_py(py),
                CompareOp::Ne => (self.bbox.left() != tuple.0
                    || self.bbox.bottom() != tuple.1
                    || self.bbox.right() != tuple.2
                    || self.bbox.top() != tuple.3)
                    .into_py(py),
                CompareOp::Lt => (self.bbox.left() < tuple.0
                    || self.bbox.bottom() < tuple.1
                    || self.bbox.right() < tuple.2
                    || self.bbox.top() < tuple.3)
                    .into_py(py),
                _ => py.NotImplemented(),
            }
        } else {
            let other = other.extract::<PyRef<PyBbox>>().unwrap();
            match op {
                CompareOp::Eq => (self.bbox.left() == other.bbox.left()
                    && self.bbox.bottom() == other.bbox.bottom()
                    && self.bbox.right() == other.bbox.right()
                    && self.bbox.top() == other.bbox.top())
                .into_py(py),
                CompareOp::Ne => (self.bbox.left() != other.bbox.left()
                    || self.bbox.bottom() != other.bbox.bottom()
                    || self.bbox.right() != other.bbox.right()
                    || self.bbox.top() != other.bbox.top())
                .into_py(py),
                CompareOp::Lt => (self.bbox.left() < other.bbox.left()
                    || self.bbox.bottom() < other.bbox.bottom()
                    || self.bbox.right() < other.bbox.right()
                    || self.bbox.top() < other.bbox.top())
                .into_py(py),
                _ => py.NotImplemented(),
            }
        }
    }
}
