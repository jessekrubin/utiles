use clap::Parser;
use tracing::{debug, warn};

/// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗      ██████╗ ███████╗██╗   ██╗
/// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝      ██╔══██╗██╔════╝██║   ██║
/// ██║   ██║   ██║   ██║██║     █████╗  ███████╗█████╗██║  ██║█████╗  ██║   ██║
/// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║╚════╝██║  ██║██╔══╝  ╚██╗ ██╔╝
/// ╚██████╔╝   ██║   ██║███████╗███████╗███████║      ██████╔╝███████╗ ╚████╔╝
///  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝      ╚═════╝ ╚══════╝  ╚═══╝
#[derive(Debug, Parser)]
#[command(name = "dev", about = "dev", long_about = "development/playground")]
pub struct DevArgs {
    #[arg(required = false)]
    fspath: Option<String>,
}
#[allow(clippy::unused_async)]
async fn dev(args: DevArgs) -> Result<(), Box<dyn std::error::Error>> {
    // DEV START
    debug!("args: {:?}", args);
    // DEV END
    Ok(())
}

pub async fn dev_main(args: DevArgs) -> Result<(), Box<dyn std::error::Error>> {
    warn!("__DEV_MAIN__");
    dev(args).await?;
    Ok(())
}
