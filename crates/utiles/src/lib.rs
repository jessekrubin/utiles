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
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
pub use core::*;
pub use errors::UtilesError;
pub use errors::UtilesResult;
pub use tile_strfmt::TileStringFormatter;

#[cfg(feature = "cli")]
pub mod cli;
mod config;
pub mod copy;
pub mod core;
pub mod dev;
pub(crate) mod errors;
pub mod fs_async;
pub mod gj;
mod globster;
pub mod img;
pub mod lager;
pub mod lint;
pub mod mbt;
mod pmt;
pub mod server;
mod signal;
pub mod sqlite;
pub mod sqlite_utiles;
mod tile_strfmt;
pub mod utilejson;

pub mod hash_types;
mod tile_stream;

pub mod cover;
pub mod edges;
pub mod hash;
mod macros;
#[cfg(test)]
mod tests;
mod timestamp;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
