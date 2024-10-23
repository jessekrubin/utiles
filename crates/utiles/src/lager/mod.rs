mod config;
mod level;

pub use config::LagerConfig;
pub use level::LagerLevel;

#[cfg(feature = "lagereload")]
mod reloadable;
#[cfg(feature = "lagereload")]
pub use reloadable::{init_tracing, set_log_format, set_log_level};

#[cfg(not(feature = "lagereload"))]
mod fixed;
#[cfg(not(feature = "lagereload"))]
pub use fixed::init_tracing;
