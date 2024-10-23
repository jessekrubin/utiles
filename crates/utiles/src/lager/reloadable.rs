use once_cell::sync::Lazy;
use std::io;
use std::sync::Mutex;
use tracing::debug;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::reload::{self, Handle};
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::errors::UtilesResult;
use crate::lager::LagerConfig;
use crate::UtilesError;

type LagerLayer = Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;
// type LagerFormatLayer = Handle<<unknown>, Layered<Box<dyn Layer<Registry>+Send+Sync>, Registry, Registry>>
type LagerFormatLayer = Handle<
    Box<
        dyn Layer<Layered<Box<dyn Layer<Registry> + Send + Sync>, Registry, Registry>>
            + Send
            + Sync,
    >,
    Layered<Box<dyn Layer<Registry> + Send + Sync>, Registry, Registry>,
>;
// Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;
pub static GLOBAL_FILTER_RELOAD_HANDLE: Lazy<Mutex<Option<LagerLayer>>> =
    Lazy::new(|| Mutex::new(None));
pub static GLOBAL_FORMAT_RELOAD_HANDLE: Lazy<Mutex<Option<LagerFormatLayer>>> =
    Lazy::new(|| Mutex::new(None));
pub static GLOBAL_LAGER_CONFIG: Lazy<Mutex<LagerConfig>> =
    Lazy::new(|| Mutex::new(LagerConfig::default()));

// #[derive(Debug, Default, Copy, Clone)]
// pub struct LagerConfig {
//     pub json: bool,
//     pub level: LagerLevel,
// }
//
// impl LagerConfig {
//     #[must_use]
//     pub fn env_filter(&self) -> EnvFilter {
//         match self.level {
//             LagerLevel::Error => EnvFilter::new("ERROR"),
//             LagerLevel::Warn => EnvFilter::new("WARN"),
//             LagerLevel::Info => EnvFilter::new("INFO"),
//             LagerLevel::Debug => EnvFilter::new("DEBUG"),
//             LagerLevel::Trace => EnvFilter::new("TRACE"),
//         }
//     }
// }

// pub fn init_tracing(log_config: &LagerConfig) -> UtilesResult<()> {
//     let filter = log_config.env_filter();
//     let (filter_layer, filter_reload_handle) = reload::Layer::new(filter.boxed());
//
//     let format_layer_raw = if log_config.json {
//         fmt::Layer::new().json().with_writer(io::stderr).boxed()
//     } else {
//         fmt::Layer::new().with_writer(io::stderr).boxed()
//     };
//     let (format_layer, format_reload_handle) = reload::Layer::new(format_layer_raw);
//     let subscriber = Registry::default()
//         .with(filter_layer.boxed())
//         .with(format_layer);
//     if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
//         debug!("tracing::subscriber::set_global_default failed: {}", e);
//     } else {
//         *GLOBAL_FILTER_RELOAD_HANDLE.lock().unwrap() = Some(filter_reload_handle);
//         *GLOBAL_FORMAT_RELOAD_HANDLE.lock().unwrap() = Some(format_reload_handle);
//     }
//     *GLOBAL_LAGER_CONFIG.lock().unwrap() = *log_config;
//     debug!("lager-config: {:?}", log_config);
//     Ok(())
// }

/// Initializes the tracing subscriber with the given logging configuration.
pub fn init_tracing(log_config: LagerConfig) -> UtilesResult<()> {
    let filter = log_config.env_filter();
    let (filter_layer, filter_reload_handle) = reload::Layer::new(filter.boxed());

    // let format_layer = if log_config.json {
    //     fmt::Layer::new().json().with_writer(io::stderr)
    // } else {
    //     fmt::Layer::new().with_writer(io::stderr)
    // };
    let format_layer_raw = if log_config.json {
        fmt::Layer::new().json().with_writer(io::stderr).boxed()
    } else {
        fmt::Layer::new().with_writer(io::stderr).boxed()
    };
    let (format_layer, format_reload_handle) = reload::Layer::new(format_layer_raw);

    let subscriber = Registry::default()
        .with(filter_layer.boxed())
        .with(format_layer);

    // Set the global default subscriber
    // tracing::subscriber::set_global_default(subscriber).map_err(|e| {
    //     UtilesError::Str(format!("Failed to set global default subscriber: {e}"))
    // })?;

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        debug!("tracing::subscriber::set_global_default failed: {}", e);
    } else {
        // *GLOBAL_FILTER_RELOAD_HANDLE.lock().unwrap() = Some(filter_reload_handle);
        // *GLOBAL_FORMAT_RELOAD_HANDLE.lock().unwrap() = Some(format_reload_handle);
        {
            let mut handle = GLOBAL_FILTER_RELOAD_HANDLE.lock().map_err(|e| {
                UtilesError::Str(format!("Failed to lock filter reload handle: {e}"))
            })?;
            *handle = Some(filter_reload_handle);
        }

        // Update global format reload handle
        {
            let mut handle = GLOBAL_FORMAT_RELOAD_HANDLE.lock().map_err(|e| {
                UtilesError::Str(format!("Failed to lock format reload handle: {e}"))
            })?;
            *handle = Some(format_reload_handle);
        }

        // Update global logging configuration
        {
            let mut config = GLOBAL_LAGER_CONFIG.lock().map_err(|e| {
                UtilesError::Str(format!("Failed to lock logging configuration: {e}"))
            })?;
            *config = log_config;
        }

        debug!("Logging configuration initialized: {:?}", log_config);
    }
    // Update global filter reload handle
    Ok(())
}
pub fn set_log_level(level: &str) -> UtilesResult<()> {
    let filter = EnvFilter::try_new(level).map_err(|e| {
        println!("failed to set log level: {e}");
        UtilesError::Str(format!("failed to set log level: {e}"))
    })?;

    let global_handle = GLOBAL_FILTER_RELOAD_HANDLE
        .lock()
        .map_err(|e| UtilesError::Str(format!("failed to lock global handle: {e}")))?;

    if let Some(handle) = global_handle.as_ref() {
        handle.reload(filter.boxed()).map_err(|e| {
            UtilesError::Str(format!("failed to reload filter layer: {e}"))
        })?;
        Ok(())
    } else {
        Err(UtilesError::Str("global reload handle not set".to_string()))
    }
}

pub fn set_log_format(json: bool) -> UtilesResult<()> {
    let format_layer_raw = if json {
        fmt::Layer::new().json().with_writer(io::stderr).boxed()
    } else {
        fmt::Layer::new().with_writer(io::stderr).boxed()
    };
    // get teh format layer reload handle
    let global_handle = GLOBAL_FORMAT_RELOAD_HANDLE
        .lock()
        .map_err(|e| UtilesError::Str(format!("failed to lock global handle: {e}")))?;
    if let Some(handle) = global_handle.as_ref() {
        handle.reload(format_layer_raw).map_err(|e| {
            UtilesError::Str(format!("failed to reload format layer: {e}"))
        })?;
        Ok(())
    } else {
        Err(UtilesError::Str("global reload handle not set".to_string()))
    }
}
