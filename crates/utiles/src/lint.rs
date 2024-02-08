use crate::utilesqlite;
use crate::utilesqlite::MbtilesAsync;
use std::path::PathBuf;
use thiserror::Error;

pub const REQUIRED_METADATA_FIELDS: [&str; 7] = [
    "name", "center", "bounds", "minzoom", "maxzoom", "format", "type",
];

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
}

pub type UtilesLintResult<T> = Result<T, UtilesLintError>;

pub struct MbtilesLinter {
    pub path: PathBuf,
    pub fix: bool,
}

impl MbtilesLinter {
    #[must_use]
    pub fn new(path: &str, fix: bool) -> Self {
        MbtilesLinter {
            path: PathBuf::from(path),
            fix,
        }
    }

    async fn open_mbtiles(
        &self,
    ) -> UtilesLintResult<utilesqlite::MbtilesAsyncSqlitePool> {
        let mbtiles = match utilesqlite::MbtilesAsyncSqlitePool::open(
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
        let magic_number = match mbt.magic_number().await {
            Ok(m) => m,
            Err(e) => {
                return Err(UtilesLintError::Unknown(e.to_string()));
            }
        };
        match magic_number {
            0 => Err(UtilesLintError::MbtMissingMagicNumber),
            _ => Err(UtilesLintError::MbtUnknownMagicNumber(magic_number)),
        }
    }

    pub async fn lint(&self) -> UtilesLintResult<Vec<UtilesLintError>> {
        let mut lint_results = vec![];
        let mbt = self.open_mbtiles().await?;
        lint_results.push(MbtilesLinter::check_magic_number(&mbt).await);
        Ok(lint_results.into_iter().filter_map(|e| e.err()).collect())
    }
}
