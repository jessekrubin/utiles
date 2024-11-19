use pyo3::prelude::*;
use pyo3::types::{PySlice, PyTuple};
use pyo3::BoundObject;

// https://users.rust-lang.org/t/solved-slice-protocol-and-custom-conversions-for-a-rust-object-exposed-to-python-via-pyo3/77633

#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(Bound<'a, PySlice>),
    Int(isize),
}

#[derive(FromPyObject, IntoPyObject)]
pub enum TupleSliceResult<T> {
    It(T),
    Slice(Vec<T>),
}
// impl<'py, T> pyo3::conversion::IntoPyObject<'py> for TupleSliceResult<T>
// where
//     T: pyo3::conversion::IntoPyObject<'py>,
//     Result<Vec<Py<pyo3::PyAny>>, pyo3::PyErr>: FromIterator<
//         Result<
//             <T as pyo3::IntoPyObject<'py>>::Output,
//             <T as pyo3::IntoPyObject<'py>>::Error,
//         >,
//     >,
// {
//     type Target = PyAny;
//     type Output = Bound<'py, Self::Target>;
//     type Error = PyErr;
//
//     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
//         match self {
//             // TupleSliceResult::It(arg) => arg
//             //     .into_pyobject(py)
//             //     .map(BoundObject::into_any)
//             //     .map(BoundObject::into_bound)
//             //     .map_err(Into::<PyErr>::into),
//             TupleSliceResult::It { 0: arg0 } => {
//                 { ::pyo3::conversion::IntoPyObject::into_pyobject(arg0, py) }
//                     .map(::pyo3::BoundObject::into_any)
//                     .map(::pyo3::BoundObject::into_bound)
//                     .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
//             }
//             TupleSliceResult::Slice(arg) => {
//                 // convert to bound tup
//                 let arg: Vec<PyObject> = arg
//                     .into_iter()
//                     .map(|x| x.into_pyobject(py))
//                     .collect::<Result<Vec<PyObject>, PyErr>>()?;
//                 let tup = PyTuple::new(py, arg);
//                 let rrrr = tup.map(BoundObject::into_any)
//                     .map(BoundObject::into_bound)
//                     .map_err(Into::<PyErr>::into);
//                 rrrr
//                 // ?.into_any();
//                 // Ok(tup.into_bound())
//
//                 // Ok(tup.into_bound(py))
//
//                 //
//                 //
//                 // arg
//                 //     .into_pyobject(py)
//                 //     .map(BoundObject::into_any)
//                 //     .map(BoundObject::into_bound)
//                 //     .map_err(Into::<PyErr>::into)
//             }
//         }
//     }
// }
// // impl<T: IntoPy<PyObject>> IntoPy<PyObject> for TupleSliceResult<T> {
// //     fn into_py(self, py: Python<'_>) -> PyObject {
// //         match self {
// //             TupleSliceResult::It(it) => it.into_py(py),
// //             TupleSliceResult::Slice(v) => {
// //                 // convert all to pyint
// //                 let v: Vec<PyObject> = v.into_iter().map(|x| x.into_py(py)).collect();
// //                 // convert to tuple
// //                 let pytuple = PyTuple::new_bound(py, v);
// //                 pytuple.into_py(py)
// //             }
// //         }
// //     }
// // }
//
// // impl<'py, T: IntoPyObject<'py> + Clone> IntoPyObject<'py> for TupleSliceResult<T> {
// //     type Target = PyAny; // the Python type
// //     type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
// //     type Error = std::convert::Infallible;
// //
// //     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
// //         match self {
// //             TupleSliceResult::It(it) => {
// //
// //                 // let v = it.into_pyobject(py);
// //                 // // downcast to pyany?
// //                 // // v.map(|x| x.into_bound(py))
// //
// //             }
// //             TupleSliceResult::Slice(v) => {
// //                 // convert all to pyint
// //                 // let v: Vec<PyObject> =
// //                 //     v.into_iter().map(|x| x.into_pyobject(py)).collect();
// //                 // let v: Vec<PyObject> =
// //                 //     v.into_iter().filter_map(|x| x.into_pyobject(py).ok()).collect();
// //                 let v = v
// //                     .into_iter()
// //                     .filter_map(|x| x.into_pyobject(py).ok())
// //                     .collect();
// //                 // convert to tuple
// //                 let pytuple = PyTuple::new(py, v);
// //                 pytuple
// //                 // Ok(pytuple.into_bound(py))
// //                 // Ok(pytuple.into_bound(py))
// //             }
// //         }
// //     }
// // }
// // //
// // // > IntoPyObject<'py> for TupleSliceResult<T>{
// // //     type Target = PyAny; // the Python type
// // //     type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
// // //     type Error = std::convert::Infallible;
// // //
// // //     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
// // //         Ok(self.0.into_bound(py))
// // //     }
// // // }
