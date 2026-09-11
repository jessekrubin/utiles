use std::cmp::Ordering;
use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyNotImplementedError, PyStopIteration};
use pyo3::prelude::*;
use utiles::BBox;

use crate::float_hash::Float64Hash;
use crate::pyutiles::pyiters::F64Iterator;
use crate::pyutiles::pytile::PyTile;

#[pyclass(name = "Bbox", module = "utiles._utiles", frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct PyBbox {
    bbox: BBox,
}

impl std::hash::Hash for PyBbox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Float64Hash::from(self.bbox.west).hash(state);
        Float64Hash::from(self.bbox.south).hash(state);
        Float64Hash::from(self.bbox.east).hash(state);
        Float64Hash::from(self.bbox.north).hash(state);
    }
}

impl PartialEq for PyBbox {
    fn eq(&self, other: &Self) -> bool {
        self.bbox.tuple() == other.bbox.tuple()
    }
}

impl PartialOrd for PyBbox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let lhs = self.bbox.tuple();
        let rhs = other.bbox.tuple();
        let pairs = [
            (lhs.0, rhs.0),
            (lhs.1, rhs.1),
            (lhs.2, rhs.2),
            (lhs.3, rhs.3),
        ];

        if pairs.iter().all(|(left, right)| left == right) {
            Some(Ordering::Equal)
        } else if pairs.iter().all(|(left, right)| left <= right)
            && pairs.iter().any(|(left, right)| left < right)
        {
            Some(Ordering::Less)
        } else if pairs.iter().all(|(left, right)| left >= right)
            && pairs.iter().any(|(left, right)| left > right)
        {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
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

    #[staticmethod]
    pub fn from_tile(tile: &PyTile) -> Self {
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

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
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

    fn __iter__(&self) -> F64Iterator {
        F64Iterator {
            iter: Box::new(
                vec![self.west(), self.south(), self.east(), self.north()].into_iter(),
            ),
        }
    }

    #[expect(clippy::unused_self, reason = "python method")]
    pub fn __len__(&self) -> usize {
        4
    }

    pub fn __getitem__(&self, idx: i32, _py: Python<'_>) -> PyResult<f64> {
        match idx {
            0 | -4 => Ok(self.bbox.left()),
            1 | -3 => Ok(self.bbox.bottom()),
            2 | -2 => Ok(self.bbox.right()),
            3 | -1 => Ok(self.bbox.top()),
            4 => Err(PyErr::new::<PyStopIteration, _>("")),
            _ => Err(PyErr::new::<PyIndexError, _>(
                "index out of range (must be -4..4)",
            )),
        }
    }

    #[expect(clippy::needless_pass_by_value, reason = "python extract")]
    fn __richcmp__(&self, other: PyBboxComparable, op: CompareOp) -> bool {
        match other {
            PyBboxComparable::Tuple(tuple) => match op {
                CompareOp::Eq => {
                    self.bbox.west() == tuple.0
                        && self.bbox.south() == tuple.1
                        && self.bbox.east() == tuple.2
                        && self.bbox.north() == tuple.3
                }
                CompareOp::Ne => {
                    self.bbox.west() != tuple.0
                        || self.bbox.south() != tuple.1
                        || self.bbox.east() != tuple.2
                        || self.bbox.north() != tuple.3
                }
                CompareOp::Lt => {
                    self.bbox.west() < tuple.0
                        || self.bbox.south() < tuple.1
                        || self.bbox.east() < tuple.2
                        || self.bbox.north() < tuple.3
                }
                CompareOp::Le => {
                    self.bbox.west() <= tuple.0
                        || self.bbox.south() <= tuple.1
                        || self.bbox.east() <= tuple.2
                        || self.bbox.north() <= tuple.3
                }
                CompareOp::Gt => {
                    self.bbox.west() > tuple.0
                        || self.bbox.south() > tuple.1
                        || self.bbox.east() > tuple.2
                        || self.bbox.north() > tuple.3
                }
                CompareOp::Ge => {
                    self.bbox.west() >= tuple.0
                        || self.bbox.south() >= tuple.1
                        || self.bbox.east() >= tuple.2
                        || self.bbox.north() >= tuple.3
                }
            },
            PyBboxComparable::PyBbox(p) => {
                let p = p.get();
                match op {
                    CompareOp::Eq => {
                        self.bbox.west() == p.bbox.west()
                            && self.bbox.south() == p.bbox.south()
                            && self.bbox.east() == p.bbox.east()
                            && self.bbox.north() == p.bbox.north()
                    }
                    CompareOp::Ne => {
                        self.bbox.west() != p.bbox.west()
                            || self.bbox.south() != p.bbox.south()
                            || self.bbox.east() != p.bbox.east()
                            || self.bbox.north() != p.bbox.north()
                    }
                    CompareOp::Lt => {
                        self.bbox.west() < p.bbox.west()
                            || self.bbox.south() < p.bbox.south()
                            || self.bbox.east() < p.bbox.east()
                            || self.bbox.north() < p.bbox.north()
                    }
                    CompareOp::Le => {
                        self.bbox.west() <= p.bbox.west()
                            || self.bbox.south() <= p.bbox.south()
                            || self.bbox.east() <= p.bbox.east()
                            || self.bbox.north() <= p.bbox.north()
                    }
                    CompareOp::Gt => {
                        self.bbox.west() > p.bbox.west()
                            || self.bbox.south() > p.bbox.south()
                            || self.bbox.east() > p.bbox.east()
                            || self.bbox.north() > p.bbox.north()
                    }
                    CompareOp::Ge => {
                        self.bbox.west() >= p.bbox.west()
                            || self.bbox.south() >= p.bbox.south()
                            || self.bbox.east() >= p.bbox.east()
                            || self.bbox.north() >= p.bbox.north()
                    }
                }
            }
        }
    }
}

enum PyBboxComparable<'a, 'py> {
    Tuple((f64, f64, f64, f64)),
    PyBbox(Borrowed<'a, 'py, PyBbox>),
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyBboxComparable<'a, 'py> {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(t) = ob.cast_exact::<pyo3::types::PyTuple>() {
            let t = t.extract::<(f64, f64, f64, f64)>()?;
            Ok(PyBboxComparable::Tuple(t))
        } else if let Ok(pybbox) = ob.cast_exact::<PyBbox>() {
            Ok(PyBboxComparable::PyBbox(pybbox))
        } else {
            Err(PyErr::new::<PyNotImplementedError, _>("Not implemented"))
        }
    }
}
