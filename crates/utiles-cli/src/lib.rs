#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
mod args;
pub mod cli;
pub mod lint;
pub mod shapes;
pub mod stdinterator;

pub use crate::cli::{cli_main, cli_main_sync};
