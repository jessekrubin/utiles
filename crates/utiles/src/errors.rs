use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesError {
    #[error("utiles-core error: {0}")]
    CoreError(#[from] utiles_core::UtilesCoreError),

    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    #[error("sqlite err: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("unknown utiles error: {0}")]
    Unknown(String),
}

pub type UtilesResult<T> = Result<T, UtilesError>;
