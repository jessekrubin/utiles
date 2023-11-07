use pyo3::{pyfunction, PyResult, Python};
use utiles_cli::cli::cli_main;

#[pyfunction]
pub fn ut_cli(py: Python, args: Option<Vec<String>>) {
    let argv = match args {
        Some(args) => args,
        None => std::env::args().collect(),
    };
    cli_main(
        Option::Some(argv),
        Option::Some(&|| {
            py.check_signals().unwrap();
        }),
    )
}
