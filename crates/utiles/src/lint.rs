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

    #[error("missing mbtiles magic-number/application_id")]
    MbtMissingMagicNumber,

    #[error("Unrecognized mbtiles magic-number/application_id: {0} != 0x4d504258")]
    MbtUnknownMagicNumber(u32),

    #[error("missing index: {0}")]
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
