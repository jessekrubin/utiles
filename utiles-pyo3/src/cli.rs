use pyo3::exceptions::PyException;
use pyo3::{pyfunction, PyResult};
use utiles::cli::cli_main_sync;

#[pyfunction]
#[pyo3(signature = (args = None))]
pub fn ut_cli(args: Option<Vec<String>>) -> PyResult<u8> {
    let argv = args.unwrap_or_else(|| std::env::args().collect());

    // clap needs the program name as first argument "utiles" or "ut"
    // ensure that the first argument is "utiles" or "ut" if not already
    // and if not, insert "utiles" as the first argument
    let utiles_argv = if let Some(p) = argv.first() {
        if p == "ut" {
            let mut v = vec!["utiles".to_string()];
            v.extend_from_slice(&argv[1..]);
            v
        } else if p == "utiles" {
            argv
        } else {
            let mut v = vec!["utiles".to_string()];
            v.extend_from_slice(&argv);
            v
        }
    } else {
        let v = vec!["utiles".to_string()];
        v
    };
    // previously we had a loop_fn argument that was a reference to a function
    // that was called in the loop, to break the cli...
    // like this: loop_fn: Option<&dyn Fn()>,
    // Some(&|| {
    //     py.check_signals().unwrap();
    // }),
    let rc = cli_main_sync(Some(utiles_argv));
    match rc {
        Ok(_) => Ok(0),
        Err(e) => {
            let py_err = PyException::new_err(format!("{e}"));
            Err(py_err)
        }
    }
}
