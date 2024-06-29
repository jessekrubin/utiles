#![cfg(test)]
#![allow(clippy::unwrap_used)]
#![deny(clippy::similar_names)]

use crate::*;
use std::collections::HashSet;

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

    let num_tiles = tiles_count(bounds, 14.into()).unwrap();
    assert_eq!(num_tiles, 2);
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
    let valid_tiles = vec![
        Tile::new(0, 0, 0),
        Tile::new(0, 0, 1),
        Tile::new(1, 1, 1),
        Tile::new(243, 166, 9),
    ];

    for tile in valid_tiles {
        assert!(tile.valid(), "{tile:?} is not valid");
    }
}

#[test]
fn tile_is_invalid() {
    let invalid_tiles = vec![
        Tile::new(0, 1, 0),
        Tile::new(1, 0, 0),
        Tile::new(1, 1, 0),
        Tile::new(1, 234, 1),
    ];

    for tile in invalid_tiles {
        assert!(!tile.valid(), "{tile:?} is valid");
    }
}

#[test]
fn test_macro() {
    let tile = utile!(0, 0, 0);
    assert_eq!(tile, Tile::new(0, 0, 0));
}

#[test]
fn test_simplify() {
    let targets = vec![
        utile!(487, 332, 10),
        utile!(486, 332, 10),
        utile!(487, 333, 10),
        utile!(973, 667, 11),
        utile!(973, 666, 11),
        utile!(972, 666, 11),
        utile!(1944, 1334, 12),
    ];

    let mut children = vec![];
    for t in &targets {
        children.extend(t.children(Some(12)));
    }
    // children.push(children[0]);
    let simplified = simplify(children.into_iter().collect::<HashSet<Tile>>());
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
