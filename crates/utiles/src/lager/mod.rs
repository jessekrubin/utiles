mod config;
mod format;
mod level;

pub use config::LagerConfig;
pub use format::LagerFormat;
pub use level::LagerLevel;

#[cfg(feature = "lager-reload")]
mod reloadable;
#[cfg(feature = "lager-reload")]
pub use reloadable::{
    get_lager_format, get_lager_level, init_tracing, set_log_format, set_log_level,
};

#[cfg(not(feature = "lager-reload"))]
mod fixed;
#[cfg(not(feature = "lager-reload"))]
pub use fixed::init_tracing;
