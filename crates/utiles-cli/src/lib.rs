#![deny(clippy::all)]
#![deny(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
mod args;
mod cli;
mod commands;
mod find;
mod stdinterator;
mod stdinterator_filter;

pub use crate::cli::{cli_main, cli_main_sync};
