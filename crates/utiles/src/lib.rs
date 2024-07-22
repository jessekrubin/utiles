#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::panic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
// road to clippy::pedantic
#![deny(clippy::pedantic)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::missing_errors_doc)]
// #![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
pub use core::*;
pub use errors::UtilesError;
pub use errors::UtilesResult;
pub use lager::init_tracing;
pub use tile_strfmt::TileStringFormatter;

pub mod cli;
mod config;
mod copy;
pub mod core;
pub mod dev;
pub(crate) mod errors;
mod fs_async;
pub mod gj;
mod globster;
mod img;
mod lager;
mod lint;
pub mod mbt;
mod pmt;
pub mod server;
mod signal;
pub mod sqlite;
pub mod sqlite_utiles;
mod tile_strfmt;
pub mod utilejson;
pub mod utilesqlite;

#[cfg(test)]
mod tests;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
