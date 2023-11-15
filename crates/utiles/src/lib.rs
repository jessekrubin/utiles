#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::style)]

use std::error::Error;
use std::collections::HashSet;
pub use lnglat::LngLat;
pub use tile::Tile;
pub use crate::fns::*;
pub use crate::quadkey::*;
pub mod bbox;
pub mod constants;
pub mod geojson;
pub mod libtiletype;
pub mod lnglat;
pub mod mbtiles;
pub mod parsing;
pub mod pmtiles;
pub mod projection;
pub mod sibling_relationship;
pub mod tile;
mod tile_feature;
pub mod tile_range;
mod tile_tuple;
pub mod tilejson;
pub mod traits;
pub mod zoom;
pub mod fns;
pub mod quadkey;

/// Tile macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to to figure out how to write a macro? yes
#[macro_export]
macro_rules! utile {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, $y, $z)
    };
}

impl From<Tile> for (u32, u32, u8) {
    fn from(tile: Tile) -> Self {
        (tile.x, tile.y, tile.z)
    }
}

// fn tile_tupless_range(
//     minx: i32,
//     maxx: i32,
//     miny: i32,
//     maxy: i32,
//     zoom_or_zooms: ZoomOrZooms,
// ) -> impl Iterator<Item = (i32, i32, i32)> {
//     let zooms = as_zooms(zoom_or_zooms);
//     zooms
//         .into_iter()
//         .flat_map(move |zoom| tiles_range_zoom(minx, maxx, miny, maxy, zoom))
// }

// fn bounds2xy(bounds: (f64, f64, f64, f64), zoom: i32) -> (i32, i32, i32, i32) {
//     let (minx, miny) = xy(bounds.0, bounds.1, None);
//     let (maxx, maxy) = xy(bounds.2, bounds.3, None);
//     let z2 = 2.0_f64.powi(zoom);
//     let minx = (minx * z2).floor() as i32;
//     let miny = (miny * z2).floor() as i32;
//     let maxx = (maxx * z2).floor() as i32;
//     let maxy = (maxy * z2).floor() as i32;
//     (minx, miny, maxx, maxy)
// }

#[cfg(test)]
mod tests {
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
