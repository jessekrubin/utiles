use async_sqlite;

use rusqlite;
use rusqlite::Result as RusqliteResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesError {
    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    #[error("invalid fspath: {0}")]
    InvalidFspath(String),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("Not a file: {0}")]
    NotAFile(String),

    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("parsing error: {0}")]
    ParsingError(String),

    #[error("utiles error: {0}")]
    Error(String),

    #[error("utiles error: {0}")]
    Str(String),

    #[error("unknown utiles error: {0}")]
    Unknown(String),

    // ===============================================================
    // EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL ~ EXTERNAL
    // ===============================================================
    /// Error from the utiles-core crate
    #[error("utiles-core error: {0}")]
    CoreError(#[from] utiles_core::UtilesCoreError),

    /// Error from `std::io`
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error from rusqlite
    #[error("sqlite err: {0}")]
    SqliteError(#[from] rusqlite::Error),

    /// Error from `async_sqlite`
    #[error("sqlite err: {0}")]
    AsyncSqliteError(#[from] async_sqlite::Error),

    /// Error from globset
    #[error("globset error: {0}")]
    GlobsetError(#[from] globset::Error),

    /// Error from `serde_json`
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub type UtilesResult<T> = Result<T, UtilesError>;

impl From<RusqliteResult<()>> for UtilesError {
    fn from(e: RusqliteResult<()>) -> Self {
        match e {
            Ok(()) => UtilesError::Unknown("unknown error".to_string()),
            Err(e) => UtilesError::SqliteError(e),
        }
    }
}
