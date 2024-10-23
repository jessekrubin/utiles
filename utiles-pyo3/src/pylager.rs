use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::{pyfunction, PyErr, PyResult};
use utiles::lager::{init_tracing, LagerConfig};

const VERSION_STRING: &'static str = "pylager";

#[pyfunction]
pub fn trace(msg: &str) {
    tracing::trace!(target: VERSION_STRING, "{}", msg);
}

#[pyfunction]
pub fn debug(msg: &str) {
    tracing::debug!(target: VERSION_STRING, "{}", msg);
}

#[pyfunction]
pub fn info(msg: &str) {
    tracing::info!(target: VERSION_STRING, "{}", msg);
}

#[pyfunction]
pub fn warn(msg: &str) {
    tracing::warn!(target: VERSION_STRING, "{}", msg);
}

#[pyfunction]
pub fn error(msg: &str) {
    tracing::error!(target: VERSION_STRING, "{}", msg);
}

#[pyfunction]
pub fn set_lager_level(level: &str) -> PyResult<()> {
    utiles::lager::set_log_level(level).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log level: {}", e))
    })
}

// #[pyfunction]
// pub fn lager_level(level: &str) -> PyResult<()> {
//     utiles::lager::set_log_level(level).map_err(|e| {
//         PyErr::new::<PyValueError, _>(format!("failed to set log level: {}", e))
//     })
// }

#[pyfunction]
pub fn set_lager_format(json: bool) -> PyResult<()> {
    utiles::lager::set_log_format(json).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log format: {}", e))
    })
}

pub fn pymod_add(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let cfg = LagerConfig::default();
    match init_tracing(cfg) {
        Ok(_) => {
            tracing::debug!("lager-config: {:?}", cfg);
        }
        Err(e) => tracing::debug!("failed to init tracing: {}", e),
    }

    m.add_function(wrap_pyfunction!(set_lager_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_lager_format, m)?)?;
    m.add_function(wrap_pyfunction!(trace, m)?)?;
    m.add_function(wrap_pyfunction!(debug, m)?)?;
    m.add_function(wrap_pyfunction!(info, m)?)?;
    m.add_function(wrap_pyfunction!(warn, m)?)?;
    m.add_function(wrap_pyfunction!(error, m)?)?;
    Ok(())
}
