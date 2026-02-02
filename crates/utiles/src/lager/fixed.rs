use std::io::{self};

use crate::errors::UtilesResult;
use crate::lager::LagerConfig;
use tracing::debug;
use tracing_subscriber::fmt::{self};

pub fn init_tracing(log_config: LagerConfig) -> UtilesResult<()> {
    let filter = log_config.env_filter();

    #[expect(clippy::match_bool)]
    #[expect(clippy::single_match_else)]
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
                .with_env_filter(filter)
                .with_writer(io::stderr)
                .with_target(log_config.is_debug_or_trace())
                .finish();
            let set_global_res = tracing::subscriber::set_global_default(subscriber);
            if let Err(e) = set_global_res {
                debug!("tracing::subscriber::set_global_default(...) failed: {}", e);
            }
        }
    }
    debug!("tracing initialized (fixed-lager)");
    debug!("lager-config: {:?}", log_config);
    Ok(())
}
