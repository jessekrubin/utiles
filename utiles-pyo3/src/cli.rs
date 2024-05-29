use pyo3::exceptions::PyException;
use pyo3::{pyfunction, PyResult, Python};

use utiles::cli::cli_main_sync;

#[pyfunction]
pub fn ut_cli(py: Python, args: Option<Vec<String>>) -> PyResult<u8> {
    let argv = args.unwrap_or_else(|| std::env::args().collect());
    let rc = cli_main_sync(
        Some(argv),
        Some(&|| {
            py.check_signals().unwrap();
        }),
    );
    match rc {
        Ok(_) => Ok(0),
        Err(e) => {
            let py_err = PyException::new_err(format!("{e}"));
            Err(py_err)
        }
    }
}
