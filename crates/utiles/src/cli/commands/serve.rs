use clap::Parser;
use tracing::debug;

use crate::server::utiles_serve;

#[derive(Debug, Parser)]
#[command(name = "dev", about = "dev", long_about = "development/playground")]
pub struct ServeArgs {
    #[arg(required = false)]
    fspaths: Option<Vec<String>>,
}

#[allow(clippy::unused_async)]
pub async fn serve_main(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("args: {:?}", args);
    utiles_serve().await.expect("utiles_serve failed");
    Ok(())
}
