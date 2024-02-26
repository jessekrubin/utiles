//! utiles-core errors mod
use thiserror::Error;

/// Error type for utiles-core
#[derive(Error, Debug)]
pub enum UtilesCoreError {
    /// Error for parsing a tile
    #[error("tile parse error: {0}")]
    TileParseError(String),

    /// Error on invalid tile-quadkey
    #[error("invalid quadkey: {0}")]
    InvalidQuadkey(String),

    /// Error for invalid bbox (bounding-box)
    #[error("invalid bbox: {0}")]
    InvalidBbox(String),

    /// Error for invalid zoom between 0 and 32
    #[error("invalid zoom(s): {0}")]
    InvalidZoom(String),

    /// Error for when converting from lnglat to web mercator fails
    #[error("conversion err: {0}")]
    LngLat2WebMercator(String),

    /// Error on unimplemented feature
    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    /// Error on unknown error catch all
    #[error("unknown utiles error: {0}")]
    Unknown(String),

    /// Error on serde io error
    #[error("io error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

/// Result type for utiles-core; really a type alias for `Result<T, UtilesCoreError>`
pub type UtilesCoreResult<T> = Result<T, UtilesCoreError>;
