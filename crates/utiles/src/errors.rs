use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesError {
    #[error("tile parse error: {0}")]
    TileParseError(String),

    #[error("invalid quadkey: {0}")]
    InvalidQuadkey(String),

    #[error("invalid bbox: {0}")]
    InvalidBbox(String),

    #[error("invalid zoom(s): {0}")]
    InvalidZoom(String),

    #[error("Coversion error: {0}")]
    ConversionError(String),

    #[error("invalid projection (must be geographic/mercator): {0}")]
    InvalidProjection(String),

    #[error("unknown utiles error")]
    Unknown,

    #[error("io error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub type UtilesResult<T> = Result<T, UtilesError>;
