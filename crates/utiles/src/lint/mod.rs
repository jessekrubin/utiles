use owo_colors::OwoColorize;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Duration;

use futures::{Stream, StreamExt, stream};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use mbt_linter::MbtilesLinter;

use crate::errors::UtilesResult;

mod mbt_linter;

pub const REQUIRED_METADATA_FIELDS: [&str; 5] =
    ["bounds", "format", "maxzoom", "minzoom", "name"];

#[derive(Error, Debug, Clone, Deserialize, Serialize, strum_macros::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum MbtLint {
    #[error("not a sqlite database error: {0}")]
    NotASqliteDb(String),

    #[error("not a mbtiles database error: {0}")]
    NotAMbtilesDb(String),

    // #[error("no tiles table/view")]
    // MbtMissingTiles,
    // #[error("no metadata table/view")]
    // MbtMissingMetadata,
    #[error("missing mbtiles magic-number/application_id")]
    MissingMagicNumber,

    #[error("Unrecognized mbtiles magic-number/application_id: {0} != 0x4d504258")]
    UnknownMagicNumber(u32),

    #[error("encoding-not-utf8: '{0}' ~ should be 'UTF-8'")]
    EncodingNotUtf8(String),

    #[error("missing index: {0}")]
    MissingIndex(String),

    #[error("missing unique index: {0}")]
    MissingUniqueIndex(String),

    #[error("duplicate metadata key: {0} (values: {1})")]
    DuplicateMetadataKey(String, String),

    #[error("duplicate metadata key-value: {0}, {1} (#{2})")]
    DuplicateMetadataKeyValue(String, String, u32),

    #[error("metadata k/v missing: {0}")]
    MissingMetadataKv(String),
}

impl MbtLint {
    #[must_use]
    pub fn format_error(&self, filepath: &str) -> String {
        let errcode = "MBT".red();
        let lint_id_ref = self.as_ref();
        let lint_id = lint_id_ref.yellow();
        let filepath_bold = filepath.bold();
        let errstr = format!("{errcode}::{lint_id} {self}");
        let e_str = format!("{filepath_bold}: {errstr}");
        e_str
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileLintResults {
    fspath: String,
    errors: Option<Vec<MbtLint>>,
    dt: Duration,
}

impl FileLintResults {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        if let Some(e) = &self.errors {
            !e.is_empty()
        } else {
            false
        }
    }

    #[must_use]
    pub fn err_str(&self) -> String {
        match &self.errors {
            Some(e) => {
                let strings = e
                    .iter()
                    .map(|e| e.format_error(&self.fspath.clone()))
                    .collect::<Vec<String>>();
                strings.join("\n")
            }
            None => String::new(),
        }
    }
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
                match lint_results {
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
                }
            }
        })
        .buffer_unordered(16)
}

pub async fn lint_filepaths(
    fspaths: Vec<PathBuf>,
    fix: bool,
) -> UtilesResult<Vec<FileLintResults>> {
    let mut results = lint_filepaths_stream(&fspaths, fix);
    let mut all_lints = Vec::new();

    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut buf = BufWriter::new(lock);

    while let Some(file_res) = results.next().await {
        if file_res.has_errors() {
            let msg = file_res.err_str();
            // println!("{msg}");
            writeln!(buf, "{msg}")?;
        }
        all_lints.push(file_res);
    }
    let n_violations = all_lints
        .iter()
        .map(
            |r: &FileLintResults| {
                if let Some(e) = &r.errors { e.len() } else { 0 }
            },
        )
        .sum::<usize>();
    // flush the buffer
    buf.flush()?;

    let msg = format!("Found {n_violations} problems");
    println!("{msg}");
    Ok(all_lints)
}
