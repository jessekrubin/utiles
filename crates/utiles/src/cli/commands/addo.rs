use crate::UtilesResult;
use crate::cli::commands::unimplemented_cmd_main;
use clap::Parser;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(name = "addo", about = "add overviews to db")]
pub(crate) struct AddoArgs {
    #[arg(required = false)]
    fspath: Option<String>,
}

pub(crate) async fn addo_main(args: Option<AddoArgs>) -> UtilesResult<()> {
    debug!("args: {:?}", args);
    unimplemented_cmd_main("addo")
}
