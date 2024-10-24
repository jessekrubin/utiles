use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::str::FromStr;
use utiles::lager::{
    get_lager_format, get_lager_level, init_tracing, LagerConfig, LagerFormat,
    LagerLevel,
};

const VERSION_STRING: &str = "pylager";

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

#[derive(Clone, Debug, PartialEq, Hash)]
#[pyclass(name = "Lager", module = "utiles._utiles")]
pub struct PyLager;

impl Default for PyLager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyLager {
    #[new]
    pub fn new() -> Self {
        Self {}
    }

    #[getter]
    fn get_level(&self) -> PyResult<String> {
        let r = get_lager_level()
            .map(|e| format!("{e}"))
            .map_err(|_e| PyErr::new::<PyValueError, _>("Failed to get lager level"))?;
        Ok(r)
    }

    #[getter]
    fn get_format(&self) -> PyResult<String> {
        let r = get_lager_format().map(|e| format!("{e}")).map_err(|_e| {
            PyErr::new::<PyValueError, _>("Failed to get lager format")
        })?;
        Ok(r)
    }

    #[setter]
    fn set_level(&self, level: &str) -> PyResult<()> {
        let parse_lev = LagerLevel::from_str(level).map_err(|_| {
            PyErr::new::<PyValueError, _>(
                "Invalid lager level ('trace', 'debug', 'info', 'warn', 'error')",
            )
        })?;
        let cur_level = get_lager_level()
            .map_err(|_e| PyErr::new::<PyValueError, _>("Failed to get lager level"))?;

        if cur_level != parse_lev {
            utiles::lager::set_log_level(level).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("failed to set log level: {e}"))
            })?;
        };
        Ok(())
    }
    #[setter]
    fn set_format(&self, fmt: &str) -> PyResult<()> {
        let parse_fmt = LagerFormat::from_str(fmt).map_err(|_e| {
            PyErr::new::<PyValueError, _>("Invalid lager level ('full', 'json')")
        })?;
        let cur_fmt = get_lager_format().map_err(|_e| {
            PyErr::new::<PyValueError, _>("Failed to get lager format")
        })?;

        if cur_fmt != parse_fmt {
            // TODO: FIX THIS NOT BE BOOL
            let is_json = cur_fmt == LagerFormat::Json;
            utiles::lager::set_log_format(is_json).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("failed to set log level: {e}"))
            })?;
        };
        Ok(())
    }

    pub fn __str__(&self) -> PyResult<String> {
        let fmt = self.get_format()?;
        let lev = self.get_level()?;
        Ok(format!("Lager(level={lev}, format={fmt})"))
    }

    pub fn trace(&self, msg: &str) {
        tracing::trace!(target: VERSION_STRING, "{}", msg);
    }

    pub fn debug(&self, msg: &str) {
        tracing::debug!(target: VERSION_STRING, "{}", msg);
    }

    pub fn info(&self, msg: &str) {
        tracing::info!(target: VERSION_STRING, "{}", msg);
    }

    pub fn warn(&self, msg: &str) {
        tracing::warn!(target: VERSION_STRING, "{}", msg);
    }

    pub fn error(&self, msg: &str) {
        tracing::error!(target: VERSION_STRING, "{}", msg);
    }
}

#[pyfunction]
pub fn set_lager_level(level: &str) -> PyResult<()> {
    utiles::lager::set_log_level(level).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log level: {e}"))
    })
}

#[pyfunction]
pub fn set_lager_format(json: bool) -> PyResult<()> {
    utiles::lager::set_log_format(json).map_err(|e| {
        PyErr::new::<PyValueError, _>(format!("failed to set log format: {e}"))
    })
}

pub fn pymod_add(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let cfg = LagerConfig::default();
    match init_tracing(cfg) {
        Ok(()) => {
            tracing::debug!("lager-config: {:?}", cfg);
        }
        Err(e) => tracing::debug!("failed to init tracing: {}", e),
    };
    let lager = PyLager::new();
    m.add_class::<PyLager>()?;
    m.add("lager", lager)?;

    m.add_function(wrap_pyfunction!(set_lager_level, m)?)?;
    m.add_function(wrap_pyfunction!(set_lager_format, m)?)?;
    m.add_function(wrap_pyfunction!(trace, m)?)?;
    m.add_function(wrap_pyfunction!(debug, m)?)?;
    m.add_function(wrap_pyfunction!(info, m)?)?;
    m.add_function(wrap_pyfunction!(warn, m)?)?;
    m.add_function(wrap_pyfunction!(error, m)?)?;
    Ok(())
}
