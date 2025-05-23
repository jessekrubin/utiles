#![doc = include_str!("../README.md")]
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
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::similar_names)]

#[doc(inline)]
pub use crate::fns::*;
#[doc(inline)]
pub use crate::merge::*;
#[doc(inline)]
pub use crate::quadkey::*;
pub use bbox::{geobbox_merge, BBox};
#[doc(inline)]
pub use errors::{UtilesCoreError, UtilesCoreResult};
#[doc(inline)]
pub use gdal::geotransform2optzoom;
pub use lnglat::{wrap_lon, LngLat};
pub use point::{Point2d, Point3d};
pub use textiles::*;
#[doc(inline)]
pub use tile::Tile;
#[doc(inline)]
pub use tile_like::TileLike;
pub use tile_strfmt::{TileStringFormat, TileStringFormatter};
pub use tile_zbox::TileZBox;
#[doc(inline)]
pub use traits::{Coord2dLike, IsOk, LngLatLike, TileChildren1, TileParent};
pub use web_geo_bounds::web_geo_bounds_union;
pub use zoom::*;
pub mod bbox;
pub mod constants;

mod errors;
pub mod fns;

pub mod gdal;
pub mod geostats;
pub mod lnglat;
pub mod parsing;

mod asserts;
mod edges;
mod macros;
mod merge;
mod parent;
#[cfg(feature = "pmtiles")]
pub mod pmtiles;
pub mod point;
pub mod projection;
pub mod quadkey;
pub mod sibling_relationship;
mod tests;
mod textiles;
pub mod tile;
pub mod tile_data_row;
mod tile_feature;
mod tile_like;
mod tile_strfmt;
mod tile_tuple;
pub mod tile_type;
pub mod tile_zbox;
mod tilecrz;
mod traits;
mod web_geo_bounds;
pub mod zoom;

pub use edges::find_edges;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::flipy;
    pub use crate::point2d;
    pub use crate::utile;
    pub use crate::utile_yup;
    pub use crate::Tile;
    pub use crate::TileLike;
}
