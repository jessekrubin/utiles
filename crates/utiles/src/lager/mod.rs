mod config;
mod format;
mod level;

pub use config::LagerConfig;
pub use format::LagerFormat;
pub use level::LagerLevel;

#[cfg(feature = "python")]
mod reloadable;
#[cfg(feature = "python")]
pub use reloadable::{
    get_lager_format, get_lager_level, init_tracing, set_log_format, set_log_level,
};

#[cfg(not(feature = "python"))]
mod fixed;
#[cfg(not(feature = "python"))]
pub use fixed::init_tracing;
