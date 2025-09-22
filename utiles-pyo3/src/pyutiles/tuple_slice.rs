use pyo3::types::PySlice;
use pyo3::{Bound, FromPyObject};

// https://users.rust-lang.org/t/solved-slice-protocol-and-custom-conversions-for-a-rust-object-exposed-to-python-via-pyo3/77633

#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(Bound<'a, PySlice>),
    Int(isize),
}
