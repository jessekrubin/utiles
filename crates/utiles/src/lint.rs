use crate::utilesqlite::MbtilesAsync;
use crate::{utilesqlite, UtilesError};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const REQUIRED_METADATA_FIELDS: [&str; 5] =
    ["bounds", "format", "maxzoom", "minzoom", "name"];
// pub const RECCOMENDED_METADATA_FIELDS: [&str; 4] = ["center", "description", "version", "attribution", "type"];

#[derive(Error, Debug)]
pub enum UtilesLintWarning {
    #[error("missing mbtiles magic-number/application_id")]
    MbtMissingMagicNumber,
}

#[derive(Error, Debug)]
pub enum UtilesLintError {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("unable to open: {0}")]
    UnableToOpen(String),

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

    #[error("unknown error: {0}")]
    Unknown(String),

    #[error("lint errors {0:?}")]
    LintErrors(Vec<UtilesLintError>),

    #[error("utiles error: {0}")]
    UtilesError(#[from] UtilesError),
}

// impl From<UtilesError> for UtilesLintError {
//     fn from(e: UtilesError) -> Self {
//         UtilesLintError::UtilesError(e)
//     }
// }

// combination of all errors and warnings
#[derive(Error, Debug)]
pub enum UtilesLint {
    #[error("error: {0}")]
    Error(#[from] UtilesLintError),

    #[error("warning: {0}")]
    Warning(#[from] UtilesLintWarning),

    #[error("lint errors {0:?}")]
    Errors(Vec<UtilesLintError>),

    #[error("lint warnings {0:?}")]
    Warnings(Vec<UtilesLintWarning>),
}

pub type UtilesLintResult<T> = Result<T, UtilesLintError>;

#[derive(Debug)]
pub struct MbtilesLinter {
    pub path: PathBuf,
    pub fix: bool,
}

impl MbtilesLinter {
    #[must_use]
    pub fn new<T: AsRef<Path>>(path: T, fix: bool) -> Self {
        MbtilesLinter {
            path: path.as_ref().to_path_buf(),
            fix,
        }
    }
    // (path: &str, fix: bool) -> Self {
    //     MbtilesLinter {
    //         path: PathBuf::from(path),
    //         fix,
    //     }
    // }

    // async fn open_mbtiles(
    //     &self,
    // ) -> UtilesLintResult<utilesqlite::MbtilesAsyncSqlitePool> {
    //     let mbtiles = match utilesqlite::MbtilesAsyncSqlitePool::open(
    //         self.path.to_str().unwrap(),
    //     )
    //         .await
    //     {
    //         Ok(m) => m,
    //         Err(e) => {
    //             return Err(UtilesLintError::UnableToOpen(e.to_string()));
    //         }
    //     };
    //     Ok(mbtiles)
    // }
    async fn open_mbtiles(
        &self,
    ) -> UtilesLintResult<utilesqlite::MbtilesAsyncSqliteClient> {
        let mbtiles = match utilesqlite::MbtilesAsyncSqliteClient::open_readonly(
            self.path.to_str().unwrap(),
        )
        .await
        {
            Ok(m) => m,
            Err(e) => {
                return Err(UtilesLintError::UnableToOpen(e.to_string()));
            }
        };
        Ok(mbtiles)
    }

    pub async fn check_magic_number<T: MbtilesAsync>(mbt: &T) -> UtilesLintResult<()> {
        let magic_number_res = mbt.magic_number().await;
        match magic_number_res {
            Ok(magic_number) => {
                if magic_number == 0x4d504258 {
                    Ok(())
                } else if magic_number == 0 {
                    Err(UtilesLintError::MbtMissingMagicNumber)
                } else {
                    Err(UtilesLintError::MbtUnknownMagicNumber(magic_number))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn check_metadata<T: MbtilesAsync>(mbt: &T) -> UtilesLintResult<()> {
        let metadata_rows = mbt.metadata_rows().await?;
        let metadata_keys = metadata_rows
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        let missing_metadata_keys = REQUIRED_METADATA_FIELDS
            .iter()
            .filter(|k| !metadata_keys.contains(&k.to_string()))
            .map(|k| k.to_string())
            .collect::<Vec<String>>();
        if missing_metadata_keys.is_empty() {
            Ok(())
        } else if missing_metadata_keys.len() == 1 {
            Err(UtilesLintError::MbtMissingMetadataKv(
                missing_metadata_keys[0].clone(),
            ))
        } else {
            let errs = missing_metadata_keys
                .iter()
                .map(|k| UtilesLintError::MbtMissingMetadataKv(k.clone()))
                .collect::<Vec<UtilesLintError>>();
            Err(UtilesLintError::LintErrors(errs))
        }
    }

    pub async fn lint(&self) -> UtilesLintResult<Vec<UtilesLintError>> {
        let mbt = self.open_mbtiles().await?;
        let mut lint_results = vec![];
        // lint_results.push(MbtilesLinter::check_magic_number(&mbt).await);

        match MbtilesLinter::check_metadata(&mbt).await {
            Ok(_) => {}
            Err(e) => match e {
                UtilesLintError::LintErrors(errs) => {
                    lint_results.push(Ok(()));
                    lint_results.extend(errs.into_iter().map(Err));
                }
                _ => {
                    lint_results.push(Err(e));
                }
            },
        }

        Ok(lint_results.into_iter().filter_map(|e| e.err()).collect())
    }
}
