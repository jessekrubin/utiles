#![deny(clippy::all)]
#![deny(clippy::perf)]
#![warn(clippy::style)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
pub mod args;
mod commands;
mod entry;
mod stdin2string;
mod stdinterator;
mod stdinterator_filter;

pub use crate::cli::entry::{cli_main, cli_main_sync, CliOpts};
