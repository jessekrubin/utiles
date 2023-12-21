// https://users.rust-lang.org/t/solved-slice-protocol-and-custom-conversions-for-a-rust-object-exposed-to-python-via-pyo3/77633
use pyo3::types::{PySlice, PyTuple};
use pyo3::{prelude::*, IntoPy, PyObject};

#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(&'a PySlice),
    Int(isize),
}

// pub enum SliceResult<T> {
//     It(T),
//     Slice(Vec<T>),
// }

// impl<T: IntoPy<PyObject>> IntoPy<PyObject> for SliceResult<T> {
//     fn into_py(self, py: Python<'_>) -> PyObject {
//         match self {
//             SliceResult::It(it) => it.into_py(py),
//             SliceResult::Slice(v) => v.into_py(py),
//         }
//     }
// }

pub enum TupleSliceResult<T> {
    // pub enum TupleSliceResult<T> {
    It(T),
    // Slice(&'a PyTuple),
    // VecSlice(Vec<T>),
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
                let pytuple = PyTuple::new(py, v);
                pytuple.into_py(py)
            }
        }
    }
}
// impl<u32: IntoPy<PyObject>> IntoPy<PyObject> for TupleSliceResult<'_,u32> {
//     // impl<u32: IntoPy<PyObject>> IntoPy<PyObject> for TupleSliceResult<u32> {
//         fn into_py(self, py: Python<'_>) -> PyObject {
//             match self {
//                 TupleSliceResult::It(it) => it.into_py(py),
//                 TupleSliceResult::Slice(v) => {
//                     let tuple = PyTuple::new(py, v);
//                     tuple.into_py(py)
//                 }
//                 TupleSliceResult::VecSlice(v) => {
//                     // convert all to pyint
//                     let mut v: Vec<PyObject> = v.into_iter().map(|x| x.into_py(py)).collect();
//                     // convert to tuple
//                     let tuple = PyTuple::new(py, v);
//                     tuple.into_py(py)
//                     // let tuple = PyTuple::new(py, v);
//                     // tuple.into_py(py)
//                 }
//             }
//         }
//     }
// pub enum TupleSliceResult<'a, T> {
// // pub enum TupleSliceResult<T> {
//     It(T),
//     Slice(&'a PyTuple),
//     VecSlice(Vec<T>),
// }
// impl<T: IntoPy<PyObject>> IntoPy<PyObject> for TupleSliceResult<'_,T> {
//     fn into_py(self, py: Python<'_>) -> PyObject {
//         match self {
//             TupleSliceResult::It(it) => it.into_py(py),
//             TupleSliceResult::Slice(v) => {
//                 let tuple = PyTuple::new(py, v);
//                 tuple.into_py(py)
//             }
//             TupleSliceResult::VecSlice(v) => {
//                 let tuple = PyTuple::new(py, v);
//                 tuple.into_py(py)
//             }
//         }
//     }
// }
