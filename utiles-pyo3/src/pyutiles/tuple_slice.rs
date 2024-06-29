use pyo3::prelude::*;
use pyo3::types::{PySlice, PyTuple};
use pyo3::{IntoPy, PyObject};

// https://users.rust-lang.org/t/solved-slice-protocol-and-custom-conversions-for-a-rust-object-exposed-to-python-via-pyo3/77633

#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(Bound<'a, PySlice>),
    Int(isize),
}

pub enum TupleSliceResult<T> {
    It(T),
    Slice(Vec<T>),
}

impl<T: IntoPy<PyObject>> IntoPy<PyObject> for TupleSliceResult<T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            TupleSliceResult::It(it) => it.into_py(py),
            TupleSliceResult::Slice(v) => {
                // convert all to pyint
                let v: Vec<PyObject> = v.into_iter().map(|x| x.into_py(py)).collect();
                // convert to tuple
                let pytuple = PyTuple::new_bound(py, v);
                pytuple.into_py(py)
            }
        }
    }
}
