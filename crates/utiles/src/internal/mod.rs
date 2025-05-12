//! repo-only (aka internal) things
pub mod cli_tools;

#[cfg(feature = "globster")]
pub(crate) mod globster;
pub mod signal;
