use crate::traits::TileChildren1;
use crate::TileParent;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Merge a set of tiles into a simplified set of tiles
///
#[must_use]
pub fn merge<T: TileParent, S: ::std::hash::BuildHasher>(
    merge_set: &HashSet<T, S>,
) -> (HashSet<T>, bool) {
    let mut upwards_merge: HashMap<T, HashSet<T>> = HashMap::new();

    for tile in merge_set {
        // let tile_parent = tile.parent(None);
        if let Some(tile_parent) = tile.parent(None) {
            upwards_merge.entry(tile_parent).or_default().insert(*tile);
        }
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
pub fn simplify_v1<T: TileParent, S: ::std::hash::BuildHasher + Default>(
    tiles: &HashSet<T, S>,
) -> HashSet<T> {
    let mut tilesv: Vec<&T> = tiles.iter().collect();
    tilesv.sort_by_key(|t| t.z());

    let mut root_set: HashSet<T> = HashSet::with_capacity(tiles.len());

    'outer: for tile in tilesv {
        for i in 0..tile.z() {
            if let Some(par) = tile.parent(Some(i)) {
                if root_set.contains(&par) {
                    continue 'outer;
                }
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

struct TileMerger<T: TileParent + TileChildren1> {
    coverage_map: HashSet<T>,
}

impl<T: TileParent + TileChildren1 + Eq + Hash + Copy + Sized> TileMerger<T> {
    fn new() -> Self {
        Self {
            coverage_map: HashSet::new(),
        }
    }

    fn has_tile_or_parent(&self, tile: &T) -> bool {
        self.coverage_map.contains(tile)
            || tile
                .iter_parents()
                .any(|el| self.coverage_map.contains(&el))
    }

    fn put(&mut self, tile: &T) -> bool {
        if self.has_tile_or_parent(tile) {
            true
        } else {
            self.coverage_map.insert(*tile);
            self.attempt_merge(*tile);
            false
        }
    }
    // fn attempt_merge(&mut self, tile: T) {
    //     tile.iter_parents().find_map(|parent_tile| {
    //         let siblings = parent_tile.children1();
    //
    //         if siblings.iter().all(|sibling| self.coverage_map.contains(sibling)) {
    //             siblings.iter().for_each(|sibling| {
    //                 self.coverage_map.remove(sibling);
    //             });
    //             self.coverage_map.insert(parent_tile);
    //             None // Continue merging, return None so the iteration continues
    //         } else {
    //             Some(()) // Stop merging, return Some to break the iteration
    //         }
    //     });
    // }

    fn attempt_merge(&mut self, tile: T) {
        for parent_tile in tile.iter_parents() {
            let siblings = parent_tile.children1();

            if siblings
                .iter()
                .all(|sibling| self.coverage_map.contains(sibling))
            {
                for sibling in &siblings {
                    self.coverage_map.remove(sibling);
                }
                self.coverage_map.insert(parent_tile);
            } else {
                // stop merging BC WE CANNOT!
                break;
            }
        }
    }
}

#[must_use]
pub fn simplify<T: TileParent + TileChildren1, S: ::std::hash::BuildHasher>(
    tiles: &HashSet<T, S>,
) -> HashSet<T> {
    let mut tiles_vec: Vec<_> = tiles.iter().collect();
    tiles_vec.sort_by_key(|a| a.z());
    let mut merger = TileMerger::new();
    for tile in tiles_vec {
        merger.put(tile);
    }
    merger.coverage_map
}
