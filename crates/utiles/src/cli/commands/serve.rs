use crate::errors::UtilesResult;
use clap::Parser;
use tracing::{debug, warn};

use crate::server::{utiles_serve, UtilesServerConfig};

#[derive(Debug, Parser)]
pub struct ServeArgs {
    /// Filesystem paths to serve from
    #[arg(required = false)]
    fspaths: Option<Vec<String>>,

    // /// config fspath (TODO)
    // #[arg(long, short = 'c')]
    // config: Option<String>,
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

#[allow(clippy::unused_async)]
pub async fn serve_main(args: ServeArgs) -> UtilesResult<()> {
    debug!("args: {:?}", args);
    if args.fspaths.is_none() || args.fspaths.as_ref().unwrap().is_empty() {
        warn!("no fspaths provided");
    }
    let cfg = args.to_cfg();
    utiles_serve(cfg).await.expect("utiles_serve failed");
    Ok(())
}
