#![deny(clippy::pedantic)]
#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::panic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![deny(clippy::complexity)]
#![deny(clippy::unwrap_used)]
#[cfg(feature = "geojson")]
mod cover_geojson;
#[cfg(feature = "geo-types")]
mod cover_geotypes;
mod errors;
#[cfg(feature = "geojson")]
pub use cover_geojson::{GeojsonCoverOptions, geojson2tiles};
#[cfg(feature = "geo-types")]
pub use cover_geotypes::{GeoTypesCoverOptions, geometry2tiles};
pub use errors::UtilesCoverError;

#[cfg(test)]
mod tests;

pub type Result<T> = std::result::Result<T, UtilesCoverError>;
