use crate::lager::LagerLevel;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Default, Copy, Clone)]
pub struct LagerConfig {
    pub json: bool,
    pub level: LagerLevel,
}

impl LagerConfig {
    #[must_use]
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
