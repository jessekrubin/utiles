use pyo3::pyfunction;
use size::Size;

#[pyfunction]
pub fn fmt_nbytes(nbytes: i64) -> String {
    let s = Size::from_bytes(nbytes);
    format!("{s}")
}
