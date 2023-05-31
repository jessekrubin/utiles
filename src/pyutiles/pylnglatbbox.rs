use crate::pyutiles::pyiters::FloatIterator;
use crate::pyutiles::pytile::PyTile;
use crate::utiles;
use crate::utiles::BBox;
use pyo3::basic::CompareOp;
use pyo3::types::PyType;
use pyo3::{
    exceptions, pyclass, pymethods, IntoPy, Py, PyAny, PyErr, PyObject, PyRef,
    PyResult, Python,
};

#[pyclass(name = "LngLatBbox")]
#[derive(Clone)]
pub struct PyLngLatBbox {
    pub bbox: BBox,
}

#[pymethods]
impl PyLngLatBbox {
    #[new]
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        PyLngLatBbox {
            bbox: BBox {
                north,
                south,
                east,
                west,
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
    pub fn from_tile(_cls: &PyType, tile: &PyTile) -> PyResult<Self> {
        let ul = utiles::ul(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        let lr = utiles::lr(tile.xyz.x, tile.xyz.y, tile.xyz.z);
        Ok(Self::new(ul.lng(), lr.lat(), lr.lng(), ul.lat()))
    }

    #[getter]
    pub fn west(&self) -> PyResult<f64> {
        Ok(self.bbox.west())
    }

    #[getter]
    pub fn south(&self) -> PyResult<f64> {
        Ok(self.bbox.south())
    }

    #[getter]
    pub fn east(&self) -> PyResult<f64> {
        Ok(self.bbox.east())
    }

    #[getter]
    pub fn north(&self) -> PyResult<f64> {
        Ok(self.bbox.north())
    }

    pub fn members(&self) -> (f64, f64, f64, f64) {
        self.tuple()
    }

    pub fn __len__(&self) -> usize {
        4
    }

    pub fn __getitem__(&self, idx: i32, _py: Python<'_>) -> PyResult<f64> {
        match idx {
            0 => Ok(self.bbox.west),
            1 => Ok(self.bbox.south),
            2 => Ok(self.bbox.east),
            3 => Ok(self.bbox.north),
            -1 => Ok(self.bbox.north),
            -2 => Ok(self.bbox.east),
            -3 => Ok(self.bbox.south),
            -4 => Ok(self.bbox.west),
            4 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),

            _ => panic!("Index {idx} out of range for tile"),
        }
    }

    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        self.bbox.tuple()
    }

    pub fn __richcmp__(
        &self,
        other: &PyAny,
        op: CompareOp,
        py: Python<'_>,
    ) -> PyObject {
        let maybetuple = other.extract::<(f64, f64, f64, f64)>();

        if let Ok(tuple) = maybetuple {
            match op {
                CompareOp::Eq => (self.bbox.west() == tuple.0
                    && self.bbox.south() == tuple.1
                    && self.bbox.east() == tuple.2
                    && self.bbox.north() == tuple.3)
                    .into_py(py),
                CompareOp::Ne => (self.bbox.west() != tuple.0
                    || self.bbox.south() != tuple.1
                    || self.bbox.east() != tuple.2
                    || self.bbox.north() != tuple.3)
                    .into_py(py),
                CompareOp::Lt => (self.bbox.west() < tuple.0
                    || self.bbox.south() < tuple.1
                    || self.bbox.east() < tuple.2
                    || self.bbox.north() < tuple.3)
                    .into_py(py),
                _ => py.NotImplemented(),
            }
        } else {
            let other = other.extract::<PyRef<PyLngLatBbox>>().unwrap();
            match op {
                CompareOp::Eq => (self.bbox.west() == other.bbox.west()
                    && self.bbox.south() == other.bbox.south()
                    && self.bbox.east() == other.bbox.east()
                    && self.bbox.north() == other.bbox.north())
                .into_py(py),
                CompareOp::Ne => (self.bbox.west != other.bbox.west()
                    || self.bbox.south() != other.bbox.south()
                    || self.bbox.east() != other.bbox.east()
                    || self.bbox.north() != other.bbox.north())
                .into_py(py),
                CompareOp::Lt => (self.bbox.west() < other.bbox.west()
                    || self.bbox.south() < other.bbox.south()
                    || self.bbox.east() < other.bbox.east()
                    || self.bbox.north() < other.bbox.north())
                .into_py(py),
                _ => py.NotImplemented(),
            }
        }
    }
}
