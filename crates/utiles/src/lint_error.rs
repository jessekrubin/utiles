use thiserror::Error;

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

    #[error("missing index: {0}")]
    MissingUniqueIndex(String),

    #[error("duplicate metadata key: {0}")]
    DuplicateMetadataKey(String),

    #[error("metadata k/v missing: {0}")]
    MbtMissingMetadataKv(String),

    #[error("unknown error")]
    Unknown,

    #[error("lint errors {0:?}")]
    LintErrors(Vec<UtilesLintError>),
}

pub type UtilesLintResult<T> = Result<T, UtilesLintError>;
