use crate::pyutiles::PyTile;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::{pyfunction, PyErr, PyResult};
use utiles::lager::{init_tracing, LagerConfig};

#[pyfunction]
pub fn debug(msg: &str) {
    tracing::debug!("{}", msg);
}
#[pyfunction]
pub fn info(msg: &str) {
    tracing::info!("{}", msg);
}

#[pyfunction]
pub fn warn(msg: &str) {
    tracing::warn!("{}", msg);
}

#[pyfunction]
pub fn error(msg: &str) {
    tracing::error!("{}", msg);
}

#[pyfunction]
pub fn trace(msg: &str) {
    tracing::trace!("{}", msg);
}

#[pyfunction]
pub fn set_lager_level(level: &str) -> PyResult<()> {
    utiles::lager::set_log_level(level).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log level: {}", e))
    })
}

#[pyfunction]
pub fn set_lager_format(json: bool) -> PyResult<()> {
    utiles::lager::set_log_format(json).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log format: {}", e))
    })
    // utiles::lager::set_log_level(level).map_err(|e| {
    //     PyErr::new::<PyValueError, _>(format!("failed to set log level: {}", e))
    // })
}

pub fn pymod_add(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let cfg = LagerConfig::default();
    let init_res = init_tracing(&cfg);
    if let Err(e) = init_res {
        tracing::warn!("failed to init tracing: {}", e);
    }

    // let mut reload_handle: Option<
    //     tracing_subscriber::reload::Handle<
    //         tracing_subscriber::EnvFilter,
    //         tracing_subscriber::Registry,
    //     >,
    // > = None;

    match init_tracing(&cfg) {
        Ok(_) => {
            tracing::debug!("lager-config: {:?}", cfg);
        }
        Err(e) => tracing::debug!("failed to init tracing: {}", e),
    }

    // add the re-load function
    // .map_err(
    //     |e| PyErr::new::<PyValueError, _>(format!("failed to init tracing: {}", e))
    // ) {
    //     return Err(res);
    // }

    m.add_function(wrap_pyfunction!(set_lager_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_lager_format, m)?)?;
    m.add_function(wrap_pyfunction!(debug, m)?)?;
    m.add_function(wrap_pyfunction!(info, m)?)?;
    m.add_function(wrap_pyfunction!(warn, m)?)?;
    m.add_function(wrap_pyfunction!(error, m)?)?;
    m.add_function(wrap_pyfunction!(trace, m)?)?;
    Ok(())
}
