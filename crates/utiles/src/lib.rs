#![warn(clippy::all)]
#![warn(clippy::correctness)]
#![warn(clippy::panic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::pedantic)]
#![warn(clippy::must_use_candidate)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_possible_wrap)]
#![expect(clippy::cast_sign_loss)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::redundant_closure_for_method_calls)]
#![expect(clippy::unnecessary_wraps)]
pub use core::*;
pub use errors::UtilesError;
pub use errors::UtilesResult;

#[macro_use]
pub mod print;

#[cfg(feature = "cli")]
pub mod cli;
mod config;
pub mod copy;
pub mod core;
pub mod dev;
pub mod errors;
pub mod fs_async;
pub mod gj;
pub mod img;
#[cfg(feature = "internal")]
pub mod internal;
#[cfg(feature = "lager")]
pub mod lager;
pub mod lint;
pub mod mbt;
mod pmt;
#[cfg(feature = "server")]
pub mod server;
pub mod sqlite;
pub mod sqlite_utiles;
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
