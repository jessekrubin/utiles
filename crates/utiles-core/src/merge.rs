use std::collections::{HashMap, HashSet};

use crate::TileParent;

/// Merge a set of tiles into a simplified set of tiles
///
#[must_use]
pub fn merge<T: TileParent, S: ::std::hash::BuildHasher + Default>(
    merge_set: &HashSet<T, S>,
) -> (HashSet<T>, bool) {
    let mut upwards_merge: HashMap<T, HashSet<T>> = HashMap::new();

    for tile in merge_set {
        let tile_parent = tile.parent(None);
        upwards_merge.entry(tile_parent).or_default().insert(*tile);
    }
    let mut current_tileset: HashSet<T> = HashSet::default();
    let mut changed = false;

    for (supertile, children) in upwards_merge {
        if children.len() == 4 {
            current_tileset.insert(supertile);
            changed = true;
        } else {
            current_tileset.extend(children);
        }
    }

    (current_tileset, changed)
}

/// Simplify a set of tiles merging children into parents
///
/// TODO: Add `minzoom` and `maxzoom` parameters
#[must_use]
pub fn simplify<T: TileParent, S: ::std::hash::BuildHasher + Default>(
    tiles: &HashSet<T, S>,
) -> HashSet<T> {
    let mut tilesv: Vec<&T> = tiles.iter().collect();
    tilesv.sort_by_key(|t| t.z());

    let mut root_set: HashSet<T> = HashSet::with_capacity(tiles.len());

    'outer: for tile in tilesv {
        for i in 0..tile.z() {
            if root_set.contains(&tile.parent(Some(i))) {
                continue 'outer;
            }
        }
        root_set.insert(*tile);
    }

    let mut is_merging = true;
    while is_merging {
        let (new_set, changed) = merge(&root_set);
        root_set = new_set;
        is_merging = changed;
    }

    root_set
}
