use clap::Parser;
use tracing::debug;

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

#[allow(clippy::unused_async)]
pub async fn serve_main(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("args: {:?}", args);
    let fspaths = vec![
        "D:\\blue-marble\\blue-marble.mbtiles".to_string(),
        // "D:\\maps\\reptiles\\mbtiles\\faacb\\20230420\\sec-crop\\Seattle_SEC_20230420_c98.mbtiles".to_string(),
        "D:\\maps\\reptiles\\mbtiles\\faacb\\20220908".to_string(),
        "D:\\maps\\reptiles\\mbtiles\\osm".to_string(),
    ];
    let cfg = UtilesServerConfig::new("0.0.0.0".to_string(), 3333, fspaths);
    utiles_serve(cfg).await.expect("utiles_serve failed");
    Ok(())
}
