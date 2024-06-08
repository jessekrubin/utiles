//! utiles-core ~ core util(e)ities
#![deny(clippy::pedantic)]
#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::panic)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::suspicious)]
#![deny(clippy::complexity)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]

pub use bbox::BBox;
pub use errors::{UtilesCoreError, UtilesCoreResult};
pub use gdal::geotransform2optzoom;
pub use lnglat::LngLat;
pub use point::{Point2d, Point3d};
pub use tile::Tile;
pub use tile_like::TileLike;
pub use tile_zbox::TileZBox;
pub use traits::{Coord2dLike, IsOk, LngLatLike};

pub use crate::fns::*;
pub use crate::quadkey::*;

pub mod bbox;
pub mod constants;
mod errors;
pub mod fns;

pub mod gdal;
pub mod geostats;
pub mod lnglat;
pub mod parsing;
pub mod pmtiles;
pub mod point;
pub mod projection;
pub mod quadkey;
pub mod sibling_relationship;
mod tests;
pub mod tile;
pub mod tile_data_row;
mod tile_feature;
mod tile_like;
mod tile_tuple;
pub mod tile_type;
pub mod tile_zbox;
mod tilecrz;
mod traits;
pub mod zoom;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Tile macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to to figure out how to write a macro? yes
#[macro_export]
macro_rules! utile {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, $y, $z)
    };
}

#[macro_export]
macro_rules! utile_yup {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, flipy($y, $z), $z)
    };
}

/// point2d macro to create a new point.
/// Replacement for coord! macro from geo-types
///
/// # Examples
///
/// ```
/// use utiles_core::{point2d, Point2d};
/// let p = point2d!{ x: 1.0, y: 2.0 };
/// assert_eq!(p.x(), 1.0);
/// assert_eq!(p.y(), 2.0);
/// ```
#[macro_export]
macro_rules! point2d {
    { x: $x:expr, y: $y:expr } => {
        Point2d::new($x, $y)
    };
}
