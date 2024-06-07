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
