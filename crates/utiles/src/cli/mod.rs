#![deny(clippy::all)]
#![deny(clippy::perf)]
#![warn(clippy::style)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::missing_panics_doc)]
#![expect(clippy::similar_names)]
#![expect(clippy::too_many_lines)]
pub mod args;
mod commands;
mod entry;
mod stdin2string;
mod stdinterator;
mod stdinterator_filter;

pub use crate::cli::entry::{cli_main, cli_main_sync, CliOpts};
