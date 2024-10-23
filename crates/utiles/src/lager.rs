use std::fmt::Formatter;
use std::io;
use std::str::FromStr;
use std::sync::Mutex;

use axum::handler::Handler;
use clap::Parser;
use futures::FutureExt;
use once_cell::sync::Lazy;
use tracing::debug;
use tracing::log::Level;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::reload::{self, Handle};
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::errors::UtilesResult;
use crate::UtilesError;
use std::sync::OnceLock;

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
static GLOBAL_FILTER_RELOAD_HANDLE: Lazy<Mutex<Option<LagerLayer>>> =
    Lazy::new(|| Mutex::new(None));
static GLOBAL_FORMAT_RELOAD_HANDLE: Lazy<Mutex<Option<LagerFormatLayer>>> =
    Lazy::new(|| Mutex::new(None));
static GLOBAL_LAGER_CONFIG: Lazy<Mutex<LagerConfig>> =
    Lazy::new(|| Mutex::new(LagerConfig::default()));

#[derive(Debug, Copy, Clone)]
pub enum LagerLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Default for LagerLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LagerLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LagerLevel::Error => write!(f, "error"),
            LagerLevel::Warn => write!(f, "warn"),
            LagerLevel::Info => write!(f, "info"),
            LagerLevel::Debug => write!(f, "debug"),
            LagerLevel::Trace => write!(f, "trace"),
        }
    }
}

impl FromStr for LagerLevel {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LagerLevel::Trace),
            "debug" => Ok(LagerLevel::Debug),
            "info" => Ok(LagerLevel::Info),
            "warn" => Ok(LagerLevel::Warn),
            "error" => Ok(LagerLevel::Error),
            _ => Err(UtilesError::Str("invalid lager level".to_string())),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LagerConfig {
    // pub debug: bool,
    // pub trace: bool,
    pub json: bool,
    pub level: LagerLevel,
}

impl LagerConfig {
    pub fn env_filter(&self) -> EnvFilter {
        match self.level {
            LagerLevel::Error => EnvFilter::new("ERROR"),
            LagerLevel::Warn => EnvFilter::new("WARN"),
            LagerLevel::Info => EnvFilter::new("INFO"),
            LagerLevel::Debug => EnvFilter::new("DEBUG"),
            LagerLevel::Trace => EnvFilter::new("TRACE"),
        }
    }
}

pub fn init_tracing(log_config: &LagerConfig) -> UtilesResult<()> {
    let filter = log_config.env_filter();
    // let filter = match log_config {
    //     LagerConfig { trace: true, .. } => EnvFilter::new("TRACE"),
    //     LagerConfig { debug: true, .. } => EnvFilter::new("DEBUG"),
    //     _ => EnvFilter::new("INFO"),
    // };
    let (filter_layer, filter_reload_handle) = reload::Layer::new(filter.boxed());
    let format_layer_raw = match log_config.json {
        true => fmt::Layer::new().json().with_writer(io::stderr).boxed(),
        false => fmt::Layer::new().with_writer(io::stderr).boxed(),
    };
    let (format_layer, format_reload_handle) = reload::Layer::new(format_layer_raw);
    let subscriber = Registry::default()
        .with(filter_layer.boxed())
        .with(format_layer);
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        debug!("tracing::subscriber::set_global_default failed: {}", e);
    } else {
        *GLOBAL_FILTER_RELOAD_HANDLE.lock().unwrap() = Some(filter_reload_handle);
        *GLOBAL_FORMAT_RELOAD_HANDLE.lock().unwrap() = Some(format_reload_handle);
    }
    *GLOBAL_LAGER_CONFIG.lock().unwrap() = log_config.clone();
    debug!("lager-config: {:?}", log_config);
    Ok(())
}

pub fn set_log_level(level: &str) -> UtilesResult<()> {
    let filter = EnvFilter::try_new(level).map_err(|e| {
        println!("failed to set log level: {}", e);
        UtilesError::Str(format!("failed to set log level: {}", e))
    })?;

    let mut global_handle = GLOBAL_FILTER_RELOAD_HANDLE.lock().map_err(|e| {
        UtilesError::Str(format!("failed to lock global handle: {}", e))
    })?;

    if let Some(handle) = global_handle.as_ref() {
        handle.reload(filter.boxed()).map_err(|e| {
            UtilesError::Str(format!("failed to reload filter layer: {}", e))
        })?;
        Ok(())
    } else {
        Err(UtilesError::Str("global reload handle not set".to_string()))
    }
}

// pub fn lager_level() -> UtilesResult<String> {
//     let logcfg =
//         GLOBAL_LAGER_CONFIG
//             .lock()
//             .map_err(Err(UtilesError::Str(String::from(
//                 "failed to get lager level",
//             ))))?;
//     Ok(logcfg.level.to_string())
// }

// pub fn set_log_format(json: bool) -> UtilesResult<()> {
//     let format_layer = match json {
//         true => fmt::Layer::default().json().with_writer(io::stderr).boxed(),
//         false => fmt::Layer::default().with_writer(io::stderr).boxed(),
//     };
//
//     let mut global_handle = GLOBAL_RELOAD_HANDLE.lock().map_err(|e| {
//         UtilesError::Str(format!("failed to lock global handle: {}", e))
//     })?;
//
//     if let Some(handle) = global_handle.as_ref() {
//         handle.reload(format_layer).map_err(|e| {
//             UtilesError::Str(format!("failed to reload format layer: {}", e))
//         })?;
//         Ok(())
//     } else {
//         Err(UtilesError::Str("global reload handle not set".to_string()))
//     }
// }
pub fn set_log_format(json: bool) -> UtilesResult<()> {
    // let format_layer = match json {
    //     true => fmt::Layer::default().json().with_writer(io::stderr).boxed(),
    //     false => fmt::Layer::default().with_writer(io::stderr).boxed(),
    // };
    let format_layer_raw = match json {
        true => fmt::Layer::new().json().with_writer(io::stderr).boxed(),
        false => fmt::Layer::new().with_writer(io::stderr).boxed(),
    };

    // get teh format layer reload handle
    let mut global_handle = GLOBAL_FORMAT_RELOAD_HANDLE
        .lock()
        .map_err(|e| UtilesError::Str(format!("failed to lock global handle: {}", e)))
        .map_err(|e| {
            UtilesError::Str(format!("failed to lock global handle: {}", e))
        })?;

    // new format layer
    // let format_layer = match global_handle.as_ref() {
    //     Some(handle) => {
    //         handle.reload(format_layer_raw).map_err(|e| {
    //             UtilesError::Str(format!("failed to reload format layer: {}", e))
    //         })?;
    //         Ok(())
    //     }
    //     None => Err(UtilesError::Str("global reload handle not set".to_string()))
    // };

    if let Some(handle) = global_handle.as_ref() {
        handle.reload(format_layer_raw).map_err(|e| {
            UtilesError::Str(format!("failed to reload format layer: {}", e))
        })?;
        Ok(())
    } else {
        Err(UtilesError::Str("global reload handle not set".to_string()))
    }

    // Ok(())
}

// use crate::errors::UtilesResult;
// use crate::UtilesError;
// use clap::Parser;
// use once_cell::sync::Lazy;
// use std::io::{self};
// use std::sync::Mutex;
// use tracing::debug;
// // use tracing_subscriber::filter::FilterExt;
// use tracing_subscriber::fmt::{self};
// use tracing_subscriber::reload::Handle;
// use tracing_subscriber::{filter, prelude::*, reload, Layer};
// use tracing_subscriber::{EnvFilter, Registry};
//
// // type LAGER_LAYER = tracing_subscriber::reload::Handle<
// //     Box<dyn tracing_subscriber::Layer<Registry> + std::marker::Send + Sync>,
// //     _,
// // >;
// type LagerLayer = Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;
// static GLOBAL_RELOAD_HANDLE: Lazy<Mutex<Option<LagerLayer>>> =
//     Lazy::new(|| Mutex::new(None));
//
// #[derive(Parser, Debug, Default)]
// pub struct LagerConfig {
//     pub debug: bool,
//     pub trace: bool,
//     pub json: bool,
// }
//
// pub fn init_tracing(log_config: &LagerConfig) -> UtilesResult<()> {
//     let filter = if log_config.trace {
//         EnvFilter::new("TRACE")
//     } else if log_config.debug {
//         EnvFilter::new("DEBUG")
//     } else {
//         EnvFilter::new("INFO")
//     };
//
//     let debug_or_trace = log_config.debug || log_config.trace;
//     // let (filter_layer, filter_reload_handle) = reload::Layer::new(filter);
//     // let format_layer = fmt::Layer::default().with_writer(io::stderr);
//     //
//     // let try_init = tracing_subscriber::registry()
//     //     .with(filter_layer)
//     //     .with(format_layer)
//     //     .with(fmt::Layer::default())
//     //     .try_init();
//     //
//     // if let Err(e) = try_init {
//     //     debug!("tracing_subscriber::registry().try_init() failed: {}", e);
//     // }
//     let filter_boxed = filter.boxed();
//
//     let (filter_layer, filter_reload_handle) = reload::Layer::new(filter_boxed);
//     let format_layer = match log_config.json {
//         // true => format_layer.json(),
//         true => fmt::Layer::default().with_writer(io::stderr).boxed(),
//         false => fmt::Layer::default().json().with_writer(io::stderr).boxed(),
//     };
//     // Initialize the subscriber with reloadable filter and formatting layer
//     let subscriber = Registry::default()
//         .with(filter_layer.boxed())
//         .with(format_layer);
//
//     // tracing::subscriber::set_global_default(subscriber).map_err(|e| {
//     //     UtilesError::Str(
//     //         format!(
//     //         "tracing_subscriber::set_global_default failed: {}",
//     //         e
//     //     )
//     //     )
//     // })?;
//     match tracing::subscriber::set_global_default(subscriber) {
//         Ok(_) => {
//             let mut global_handle = GLOBAL_RELOAD_HANDLE.lock().unwrap();
//             *global_handle = Some(filter_reload_handle);
//         }
//         Err(e) => {
//             debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
//         }
//     }
//     debug!("lager-config: {:?}", log_config);
//     // Set the global reload handle
//     Ok(())
//     // Ok(filter_reload_handle)
// }
//
// pub fn set_log_level(level: &str) -> UtilesResult<()> {
//     println!("set_log_level: {}", level);
//     let filter = EnvFilter::try_new(level).map_err(|e| {
//         println!("failed to set log level: {}", e);
//         UtilesError::Str(format!("failed to set log level: {}", e))
//     })?;
//
//     let mut global_handle = GLOBAL_RELOAD_HANDLE.lock().map_err(|e| {
//         UtilesError::Str(format!("failed to lock global handle: {}", e))
//     })?;
//     let a = global_handle.as_ref();
//     if let Some(handle) = a {
//         let f = filter.boxed();
//         handle.reload(f);
//         Ok(())
//     } else {
//         Err(UtilesError::Str("global reload handle not set".to_string()))
//     }
// }
//
// pub fn set_log_format(json: bool) -> UtilesResult<()> {
//     let format_layer = match json {
//         true => fmt::Layer::default().with_writer(io::stderr).boxed(),
//         false => fmt::Layer::default().json().with_writer(io::stderr).boxed(),
//     };
//     let mut global_handle = GLOBAL_RELOAD_HANDLE.lock().map_err(|e| {
//         UtilesError::Str(format!("failed to lock global handle: {}", e))
//     })?;
//     let a = global_handle.as_ref();
//     if let Some(handle) = a {
//         handle.reload(format_layer);
//         Ok(())
//     } else {
//         Err(UtilesError::Str("global reload handle not set".to_string()))
//     }
// }
// //
// //
// // pub fn init_tracing(log_config: &LagerConfig) -> UtilesResult<()> {
// //     let filter = if log_config.trace {
// //         EnvFilter::new("TRACE")
// //     } else if log_config.debug {
// //         EnvFilter::new("DEBUG")
// //     } else {
// //         EnvFilter::new("INFO")
// //     };
// //     let debug_or_trace = log_config.debug || log_config.trace;
// //
// //
// //
// //     #[allow(clippy::match_bool)]
// //     match log_config.json {
// //         true => {
// //             let subscriber = fmt::Subscriber::builder()
// //                 .json()
// //                 .with_env_filter(filter)
// //                 .with_writer(io::stderr)
// //                 .finish();
// //             let set_global_res = tracing::subscriber::set_global_default(subscriber);
// //             if let Err(e) = set_global_res {
// //                 debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
// //             }
// //         }
// //         false => {
// //             let subscriber = fmt::Subscriber::builder()
// //                 .with_env_filter(filter)
// //                 .with_writer(io::stderr)
// //                 .with_target(debug_or_trace)
// //                 .finish();
// //             let set_global_res = tracing::subscriber::set_global_default(subscriber);
// //             if let Err(e) = set_global_res {
// //                 debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
// //             }
// //         }
// //     }
// //     debug!("lager-config: {:?}", log_config);
// //     Ok(())
// // }
