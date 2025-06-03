use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesCopyError {
    #[error("src and dst: {0}")]
    SrcDstSame(String),

    #[error("src does not exist: {0}")]
    SrcNotExists(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

#[derive(Error, Debug)]
pub enum UtilesError {
    #[error("utiles error: {0}")]
    AdHoc(String),

    /// Error for unimplemented feature(s) with a string message for context
    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    /// unsupported err for when a feature is not supported
    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Unknown filetype: {0}")]
    UnknownFiletype(String),

    #[error("invalid fspath: {0}")]
    InvalidFspath(String),

    #[error("No fspath extension: {0}")]
    NoFspathExtension(String),

    #[error("No fspath stem: {0}")]
    NoFspathStem(String),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("metadata error: {0}")]
    MetadataError(String),

    #[error("Path already exists: {0}")]
    PathExistsError(String),

    #[error("Not a file: {0}")]
    NotAFile(String),

    #[error("Non mbtiles sqlite db: {0}")]
    NonMbtilesSqliteDb(String),

    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("parsing error: {0}")]
    ParsingError(String),

    #[error("Not mbtiles-like: {0}")]
    NotMbtilesLike(String),

    #[error("utiles error: {0}")]
    Error(String),

    #[error("unknown utiles error: {0}")]
    Unknown(String),

    #[error("path conversion error: {0}")]
    PathConversionError(String),

    /// simple string lock error type that might be good to fix up
    #[error("Lock error: {0}")]
    LockError(String),

    // ===============================================================
    // EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL
    // ===============================================================
    /// Error from the utiles-core crate
    #[error("utiles-core error: {0}")]
    CoreError(#[from] utiles_core::UtilesCoreError),

    #[error("utiles-cover error: {0}")]
    CoverError(#[from] utiles_cover::UtilesCoverError),

    /// Error from `utiles::copy`
    #[error("utiles-copy error: {0}")]
    CopyError(#[from] UtilesCopyError),

    /// Error from `serde_json`
    #[error("serde error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    /// Error from `std::io`
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error from `sqlite` module
    #[error("{0}")]
    SqliteError(#[from] crate::sqlite::SqliteError),

    /// Error from rusqlite
    #[error("rusqlite err: {0}")]
    RusqliteError(#[from] rusqlite::Error),

    /// Error from `async_sqlite`
    #[error("sqlite err: {0}")]
    AsyncSqliteError(#[from] async_sqlite::Error),

    /// Geojson error(s)
    #[error("geojson error: {0}")]
    GeojsonError(String),

    /// Error from globset
    #[cfg(feature = "globster")]
    #[error("globset error: {0}")]
    GlobsetError(#[from] globset::Error),

    /// Image error
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),

    /// Error from `json_patch`
    #[error("json_patch error: {0}")]
    JsonPatchError(#[from] json_patch::PatchError),

    /// Error from `tokio::task`
    #[error("tokio::task::JoinError - {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    /// Error from `pmtiles`
    #[cfg(feature = "pmtiles")]
    #[error("pmtiles error: {0}")]
    PmtilesError(#[from] pmtiles::PmtError),
    // /// Error from `oxipng`
    // #[error("oxipng::PngError: {0}")]
    // OxipngError(#[from] oxipng::PngError),

    // /// ndarray shape error
    // #[error("ndarray shape error: {0}")]
    // NdarrayShapeError(#[from] ndarray::ShapeError),
}

pub type UtilesResult<T, E = UtilesError> = Result<T, E>;
