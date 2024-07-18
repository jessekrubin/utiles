use std::path::PathBuf;
use std::time::Duration;

use colored::Colorize;
use futures::{stream, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use mbt_linter::MbtilesLinter;

use crate::UtilesResult;

mod mbt_linter;

pub const REQUIRED_METADATA_FIELDS: [&str; 5] =
    ["bounds", "format", "maxzoom", "minzoom", "name"];

#[derive(Error, Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileLintResults {
    fspath: String,
    errors: Option<Vec<UtilesLintError>>,
    dt: Duration,
}
pub fn lint_filepaths_stream(
    fspaths: &Vec<PathBuf>,
    fix: bool,
) -> impl Stream<Item = FileLintResults> + '_ {
    stream::iter(fspaths)
        .map(move |path| {
            let linter = MbtilesLinter::new(path, fix);
            async move {
                debug!("linting: {}", path.display());
                let start_time = std::time::Instant::now();

                let lint_results = linter.lint().await;
                let elapsed = start_time.elapsed();
                let file_results = match lint_results {
                    Ok(r) => FileLintResults {
                        fspath: path.display().to_string(),
                        errors: Some(r),
                        dt: elapsed,
                    },
                    Err(e) => {
                        warn!("lint error: {}", e);
                        FileLintResults {
                            fspath: path.display().to_string(),
                            errors: None,
                            dt: elapsed,
                        }
                    }
                };
                file_results
            }
        })
        .buffer_unordered(12)
}
pub async fn lint_filepaths(
    fspaths: Vec<PathBuf>,
    fix: bool,
) -> UtilesResult<Vec<FileLintResults>> {
    let mut results = lint_filepaths_stream(&fspaths, fix);
    let all_lints = Vec::new();
    // let mut errors = Vec::new();
    while let Some(file_res) = results.next().await {
        // let json_string = serde_json::to_string(&file_res).unwrap();
        // println!("{json_string}");
        match file_res.errors {
            Some(r) => {
                debug!("r: {:?}", r);
                if r.is_empty() {
                    debug!("OK: {}", file_res.fspath);
                } else {
                    debug!("{} - {} errors found", file_res.fspath, r.len());
                    let strings = r
                        .iter()
                        .map(|e| e.format_error(&file_res.fspath.to_string()))
                        .collect::<Vec<String>>();
                    let joined = strings.join("\n");
                    println!("{joined}");

                    for err in &r {
                        debug!("{}", err.to_string());
                    }
                }
            }
            None => {
                debug!("OK: {}", file_res.fspath);
            }
        }
    }
    Ok(all_lints)
}
