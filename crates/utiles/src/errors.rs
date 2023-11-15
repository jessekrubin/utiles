use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesErrors {
    #[error("invalid quadkey: {0}")]
    InvalidQuadkey(String),

    #[error("unknown utiles error")]
    Unknown,
}

pub type UtilesResult<T> = Result<T, UtilesLintError>;
