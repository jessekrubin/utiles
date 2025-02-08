use crate::cli::commands::unimplemented_cmd_main;
use crate::UtilesResult;
use clap::Parser;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(name = "translate", about = "translate/convert files")]
pub(crate) struct TranslateArgs {
    #[arg(required = false)]
    fspath: Option<String>,
}

pub(crate) async fn translate_main(args: Option<TranslateArgs>) -> UtilesResult<()> {
    debug!("args: {:?}", args);
    unimplemented_cmd_main("translate")
}
