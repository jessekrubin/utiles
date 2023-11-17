use pyo3::{pyfunction, Python};
use utiles_cli::cli_main_sync;

#[pyfunction]
pub fn ut_cli(py: Python, args: Option<Vec<String>>) -> u8 {
    let argv = match args {
        Some(args) => args,
        None => std::env::args().collect(),
    };
    cli_main_sync(
        Some(argv),
        Some(&|| {
            py.check_signals().unwrap();
        }),
    )
}
