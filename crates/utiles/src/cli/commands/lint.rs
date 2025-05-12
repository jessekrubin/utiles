use tracing::{debug, warn};

use crate::cli::args::LintArgs;
use crate::errors::UtilesResult;
use crate::internal::globster;
use crate::lint::lint_filepaths;

pub(crate) async fn lint_main(args: &LintArgs) -> UtilesResult<()> {
    let filepaths = globster::find_filepaths(&args.fspaths)?;
    if args.fix {
        warn!("NOT IMPLEMENTED: `utiles lint --fix`");
    }
    debug!("filepaths: {:?}", filepaths);
    if filepaths.is_empty() {
        warn!("No files found");
        return Ok(());
    }
    lint_filepaths(filepaths, args.fix).await?;
    Ok(())
}
