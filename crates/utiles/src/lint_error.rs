use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilesLintError {
    #[error("not a sqlite database error: {0}")]
    NotASqliteDb(String),

    #[error("unknown data store error")]
    Unknown,
}
// use thiserror::Error;
//
// #[derive(Error, Debug)]
// pub enum DataStoreError {
//     #[error("data store disconnected")]
//     Disconnect(#[from] io::Error),
//     #[error("the data for key `{0}` is not available")]
//     Redaction(String),
//     #[error("invalid header (expected {expected:?}, found {found:?})")]
//     InvalidHeader {
//         expected: String,
//         found: String,
//     },
//     #[error("unknown data store error")]
//     Unknown,
// }

pub type UtilesLintResult<T> = Result<T, UtilesLintError>;
