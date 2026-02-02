use clap::Parser;
use tracing::{debug, warn};

use crate::errors::UtilesResult;
use crate::server::{UtilesServerConfig, utiles_serve};

#[derive(Debug, Parser)]
pub struct ServeArgs {
    /// Filesystem paths to serve from
    #[arg(required = false)]
    fspaths: Option<Vec<String>>,

    /// Port to server on
    #[arg(long, short = 'p', default_value = "3333")]
    port: u16,

    /// Host bind address
    #[arg(long, short = 'H', default_value = "0.0.0.0")]
    host: String,

    /// strict mode (default: false)
    #[arg(long, short = 's', default_value = "false", action = clap::ArgAction::SetTrue)]
    strict: bool,
}

impl ServeArgs {
    pub fn to_cfg(&self) -> UtilesServerConfig {
        UtilesServerConfig::new(
            self.host.clone(),
            self.port,
            self.fspaths.clone().unwrap_or_default(),
        )
    }
}

pub(crate) async fn serve_main(args: ServeArgs) -> UtilesResult<()> {
    debug!("args: {:?}", args);
    if let Some(ref fspaths) = args.fspaths {
        if fspaths.is_empty() {
            warn!("fspaths is empty");
        }
        for fspath in fspaths {
            if !std::path::Path::new(fspath).exists() {
                warn!("fspath does not exist: {:?}", fspath);
            }
        }
    } else {
        warn!("no fspaths provided");
    }
    let cfg = args.to_cfg();
    utiles_serve(cfg).await?;
    Ok(())
}
