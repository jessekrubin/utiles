use async_sqlite;

use rusqlite;
use rusqlite::Result as RusqliteResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesError {
    #[error("utiles-core error: {0}")]
    CoreError(#[from] utiles_core::UtilesCoreError),

    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    #[error("sqlite err: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("sqlite err: {0}")]
    AsyncSqliteError(#[from] async_sqlite::Error),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("utiles error: {0}")]
    Error(String),

    #[error("unknown utiles error: {0}")]
    Unknown(String),
}

pub type UtilesResult<T> = Result<T, UtilesError>;

impl From<RusqliteResult<()>> for UtilesError {
    fn from(e: RusqliteResult<()>) -> Self {
        match e {
            Ok(_) => UtilesError::Unknown("unknown error".to_string()),
            Err(e) => UtilesError::SqliteError(e),
        }
    }
}
