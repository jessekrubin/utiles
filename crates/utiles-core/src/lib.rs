#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::correctness)]
#![warn(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]

pub use gdal::geotransform2optzoom;
pub use lnglat::LngLat;
pub use point::{Point2d, Point3d};
pub use tile::Tile;
pub use tile_like::TileLike;

pub use crate::errors::UtilesCoreError;
pub use crate::fns::*;
pub use crate::quadkey::*;

pub mod errors;

pub mod bbox;
pub mod constants;
pub mod fns;
pub mod gdal;
pub mod geostats;
pub mod lnglat;
pub mod mbutiles;
pub mod parsing;
pub mod pmtiles;
pub mod point;
pub mod projection;
pub mod quadkey;
pub mod sibling_relationship;
pub mod tile;
pub mod tile_data_row;
mod tile_feature;
mod tile_like;
pub mod tile_range;
mod tile_tuple;
pub mod tile_type;
mod tilecrz;
pub mod traits;
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn zoom_or_zooms() {
        let z = as_zooms(1.into());
        assert_eq!(z, vec![1]);
        let z = as_zooms(vec![1, 2, 3].into());
        assert_eq!(z, vec![1, 2, 3]);
    }

    #[test]
    fn tiles_generator() {
        let bounds = (-105.0, 39.99, -104.99, 40.0);
        let tiles = tiles(bounds, vec![14].into());
        let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
        assert_eq!(tiles.collect::<Vec<Tile>>(), expect);
    }

    #[test]
    fn tiles_single_zoom() {
        let bounds = (-105.0, 39.99, -104.99, 40.0);
        let tiles = tiles(bounds, 14.into());
        let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
        assert_eq!(tiles.collect::<Vec<Tile>>(), expect);

        let ntiles = tiles_count(bounds, 14.into());
        assert_eq!(ntiles, 2);
    }

    #[test]
    fn tiles_anti_meridian() {
        let bounds = (175.0, 5.0, -175.0, 10.0);
        let mut tiles: Vec<Tile> = tiles(bounds, 2.into()).collect();
        tiles.sort();
        let mut expected = vec![Tile::new(3, 1, 2), Tile::new(0, 1, 2)];
        expected.sort();
        assert_eq!(tiles, expected);
    }

    #[test]
    fn tile_is_valid() {
        let tile = Tile::new(0, 0, 0);
        assert!(tile.valid());
        let tile = Tile::new(1, 0, 0);
        assert!(!tile.valid());
        let tile = Tile::new(0, 1, 0);
        assert!(!tile.valid());
        let tile = Tile::new(0, 0, 1);
        assert!(tile.valid());
        let tile = Tile::new(1, 1, 1);
        assert!(tile.valid());

        // invalid tile
        let tile = Tile::new(1, 1, 0);
        assert!(!tile.valid());
        let _tile = Tile::new(1, 234, 1);
        assert!(!_tile.valid());
    }

    #[test]
    fn test_macro() {
        let tile = utile!(0, 0, 0);
        assert_eq!(tile, Tile::new(0, 0, 0));
    }

    #[test]
    fn test_simplify() {
        let children = utile!(243, 166, 9).children(Some(12));
        assert_eq!(children.len(), 64);
        let mut children = children.into_iter().collect::<Vec<Tile>>();
        children.truncate(61);
        children.push(children[0]);
        let simplified = simplify(children.into_iter().collect::<HashSet<Tile>>());
        let targets = vec![
            utile!(487, 332, 10),
            utile!(486, 332, 10),
            utile!(487, 333, 10),
            utile!(973, 667, 11),
            utile!(973, 666, 11),
            utile!(972, 666, 11),
            utile!(1944, 1334, 12),
        ];
        for target in targets {
            assert!(simplified.contains(&target));
        }
    }

    #[test]
    fn test_simplify_removal() {
        let tiles = vec![
            utile!(1298, 3129, 13),
            utile!(649, 1564, 12),
            utile!(650, 1564, 12),
        ];
        let simplified = simplify(tiles.into_iter().collect::<HashSet<Tile>>());
        assert!(!simplified.contains(&utile!(1298, 3129, 13)));
        assert!(simplified.contains(&utile!(650, 1564, 12)));
        assert!(simplified.contains(&utile!(649, 1564, 12)));
    }
}
