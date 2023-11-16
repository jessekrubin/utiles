#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
mod args;
mod cli;
mod lint;
mod shapes;
mod stdinterator;

pub use crate::cli::{cli_main, cli_main_sync};
