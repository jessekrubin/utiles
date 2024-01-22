use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesCoreError {
    #[error("tile parse error: {0}")]
    TileParseError(String),

    #[error("invalid quadkey: {0}")]
    InvalidQuadkey(String),

    #[error("invalid bbox: {0}")]
    InvalidBbox(String),

    #[error("invalid zoom(s): {0}")]
    InvalidZoom(String),

    #[error("invalid tile dimensions(s): {0}")]
    InvalidTileDim(String),

    #[error("Coversion error: {0}")]
    ConversionError(String),

    #[error("invalid projection (must be geographic/mercator): {0}")]
    InvalidProjection(String),

    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    #[error("sqlite err: {0}")]
    SqliteErr(String),

    #[error("unknown utiles error: {0}")]
    Unknown(String),

    #[error("io error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub type UtilesCoreResult<T> = Result<T, UtilesCoreError>;
