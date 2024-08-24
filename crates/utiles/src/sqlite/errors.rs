use async_sqlite;
use rusqlite;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqliteError {
    #[error("Invalid page size: {0} must be power of 2 and between 512 and 65536")]
    InvalidPageSize(String),

    /// Error from rusqlite
    #[error("rusqlite err: {0}")]
    RusqliteError(#[from] rusqlite::Error),

    /// Error from `async_sqlite`
    #[allow(clippy::enum_variant_names)]
    #[error("sqlite err: {0}")]
    AsyncSqliteError(async_sqlite::Error),

    /// Deadpool interact error
    #[error("Deadpool interact error: {0}")]
    DeadpoolInteractErrorPanic(String),

    /// Deadpool PoolError
    #[error("Deadpool pool error: {0}")]
    DeadpoolPoolError(#[from] deadpool::managed::PoolError<rusqlite::Error>),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    /// Error from `std::io`
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid sqlite-magic; must be `53 51 4c 69 74 65 20 66 6f 72 6d 61 74 20 33 00`
    #[error("Invalid sqlite-magic: {0}")]
    InvalidSqliteMagic(String),

    /// Parse header field
    #[error("Invalid header field: {0}")]
    ParseHeaderField(String),

    /// Invalid header field
    #[error("Invalid header field: {0}")]
    InvalidHeaderField(String),

    /// Invalid sqlite db
    #[error("Invalid sqlite db: {0}")]
    InvalidSqliteDb(String),
}

pub type SqliteResult<T> = Result<T, SqliteError>;

impl From<async_sqlite::Error> for SqliteError {
    fn from(value: async_sqlite::Error) -> Self {
        match value {
            async_sqlite::Error::Rusqlite(value) => SqliteError::RusqliteError(value),
            _ => SqliteError::AsyncSqliteError(value),
        }
    }
}

impl From<deadpool_sync::InteractError> for SqliteError {
    fn from(value: deadpool_sync::InteractError) -> Self {
        match value {
            deadpool_sync::InteractError::Panic(value) => {
                SqliteError::DeadpoolInteractErrorPanic(format!("{value:?}"))
            }
            _ => SqliteError::DeadpoolInteractErrorPanic("Aborted".to_string()),
        }
    }
}
