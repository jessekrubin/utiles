use tracing_subscriber::{EnvFilter, fmt};
use std::io;

pub struct LogConfig {
    pub debug: bool,
    pub json: bool,
}

pub fn init_tracing(log_config: LogConfig) {
    let filter = if log_config.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("INFO")
    };
    match log_config.json {
        true => {
            let subscriber = fmt::Subscriber::builder()
                .json()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default(...) failed.");
        }
        false => {
            let subscriber = fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default(...) failed.");
        }
    }
}
