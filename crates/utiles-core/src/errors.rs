//! utiles-core errors mod

use std::num::TryFromIntError;
use thiserror::Error;

/// Error type for utiles-core
#[derive(Error, Debug)]
pub enum UtilesCoreError {
    /// Error with some string
    #[error("{0}")]
    AdHoc(String),

    /// Error for parsing a tile
    #[error("tile parse error: {0}")]
    TileParseError(String),

    /// Error for general parsing
    #[error("parse error: {0}")]
    ParseError(String),

    /// Error on invalid tile-quadkey
    #[error("invalid tile: {0}")]
    InvalidTile(String),

    /// Error on invalid tile-quadkey
    #[error("invalid quadkey: {0}")]
    InvalidQuadkey(String),

    /// Error for invalid bbox (bounding-box)
    #[error("invalid bbox: {0}")]
    InvalidBbox(String),

    /// Error for invalid `LngLat`
    #[error("invalid lnglat: {0}")]
    InvalidLngLat(String),

    /// Error for invalid SRTM string
    #[error("invalid SRTM string: {0}")]
    InvalidSrtmString(String),

    /// Error for invalid zoom between 0 and 32
    #[error("invalid zoom(s): {0}")]
    InvalidZoom(String),

    /// Error for invalid projection
    #[error("invalid projection: {0}")]
    InvalidProjection(String),

    /// Error for invalid json
    #[error("invalid json: {0}")]
    InvalidJson(String),

    /// Error for when converting from lnglat to web mercator fails
    #[error("conversion err: {0}")]
    LngLat2WebMercator(String),

    /// Error on unimplemented feature
    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    /// Error on serde io error
    #[error("io error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Try from int
    #[error("try-from-int: {0}")]
    TryFromIntError(#[from] TryFromIntError),
}

/// Result type for utiles-core; really a type alias for `Result<T, UtilesCoreError>`
pub type UtilesCoreResult<T> = Result<T, UtilesCoreError>;
