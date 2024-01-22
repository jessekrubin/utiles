use pyo3::{pyfunction, Python};
use utiles::cli::cli_main_sync;

#[pyfunction]
pub fn ut_cli(py: Python, args: Option<Vec<String>>) -> u8 {
    let argv = args.unwrap_or_else(|| std::env::args().collect());
    cli_main_sync(
        Some(argv),
        Some(&|| {
            py.check_signals().unwrap();
        }),
    )
}
