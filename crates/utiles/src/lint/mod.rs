use std::path::PathBuf;

use colored::Colorize;
use futures::{stream, StreamExt};
use thiserror::Error;
use tracing::{debug, warn};

use mbt_linter::MbtilesLinter;

mod mbt_linter;

pub const REQUIRED_METADATA_FIELDS: [&str; 5] =
    ["bounds", "format", "maxzoom", "minzoom", "name"];

#[derive(Error, Debug)]
pub enum UtilesLintError {
    #[error("not a sqlite database error: {0}")]
    NotASqliteDb(String),

    #[error("not a mbtiles database error: {0}")]
    NotAMbtilesDb(String),

    #[error("no tiles table/view")]
    MbtMissingTiles,

    #[error("no metadata table/view")]
    MbtMissingMetadata,

    #[error("missing mbtiles magic-number/application_id")]
    MbtMissingMagicNumber,

    #[error("Unrecognized mbtiles magic-number/application_id: {0} != 0x4d504258")]
    MbtUnknownMagicNumber(u32),

    #[error("missing index: {0}")]
    MissingIndex(String),

    #[error("missing unique index: {0}")]
    MissingUniqueIndex(String),

    #[error("duplicate metadata key: {0}")]
    DuplicateMetadataKey(String),

    #[error("metadata k/v missing: {0}")]
    MbtMissingMetadataKv(String),

    #[error("lint errors {0:?}")]
    LintErrors(Vec<UtilesLintError>),
}

impl UtilesLintError {
    #[must_use]
    pub fn format_error(&self, filepath: &str) -> String {
        let errcode = "MBT".red();
        let errstr = format!("{errcode}: {self}");
        let e_str = format!("{filepath}: {errstr}");
        e_str
    }
}
pub type UtilesLintResult<T> = Result<T, UtilesLintError>;

pub async fn lint_filepaths(fspaths: Vec<PathBuf>, fix: bool) {
    stream::iter(fspaths)
        .for_each_concurrent(4, |path| async move {
            let linter = MbtilesLinter::new(&path, fix);
            let lint_results = linter.lint().await;
            match lint_results {
                Ok(r) => {
                    debug!("r: {:?}", r);
                    // print each err....
                    if r.is_empty() {
                        debug!("OK: {}", path.display());
                    } else {
                        debug!("{} - {} errors found", path.display(), r.len());
                        // let agg_err = UtilesLintError::LintErrors(r);
                        let strings = r
                            .iter()
                            .map(|e| e.format_error(&path.display().to_string()))
                            .collect::<Vec<String>>();
                        let joined = strings.join("\n");
                        println!("{joined}");
                        for err in r {
                            debug!("{}", err.to_string());
                        }
                    }
                }
                Err(e) => {
                    let e_msg = format!("{}: {}", path.display(), e);
                    warn!("{}", e_msg);
                }
            }
        })
        .await;
}
