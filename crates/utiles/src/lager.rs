use std::io::{self};

use clap::Parser;
use tracing::debug;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::EnvFilter;

use crate::errors::UtilesResult;

#[derive(Parser, Debug)]
pub struct LagerConfig {
    pub debug: bool,
    pub trace: bool,
    pub json: bool,
}
impl Default for LagerConfig {
    fn default() -> Self {
        Self {
            debug: false,
            trace: false,
            json: false,
        }
    }
}

pub fn init_tracing(log_config: &LagerConfig) -> UtilesResult<()> {
    let filter = if log_config.trace {
        EnvFilter::new("TRACE")
    } else if log_config.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("INFO")
    };
    let debug_or_trace = log_config.debug || log_config.trace;

    #[allow(clippy::match_bool)]
    match log_config.json {
        true => {
            let subscriber = fmt::Subscriber::builder()
                .json()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .finish();
            let set_global_res = tracing::subscriber::set_global_default(subscriber);
            if let Err(e) = set_global_res {
                debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
            }
        }
        false => {
            let subscriber = fmt::Subscriber::builder()
                .without_time()
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .with_target(debug_or_trace)
                .finish();
            let set_global_res = tracing::subscriber::set_global_default(subscriber);
            if let Err(e) = set_global_res {
                debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
            }
        }
    }
    debug!("lager-config: {:?}", log_config);
    Ok(())
}
