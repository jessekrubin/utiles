use std::collections::{HashMap, HashSet};

use pyo3::types::PyTuple;
use pyo3::{pyfunction, Bound, PyResult};

use crate::pyutiles::pyparsing::parse_tiles;
use crate::pyutiles::pytile::PyTile;

fn merge(merge_set: &HashSet<PyTile>) -> (HashSet<PyTile>, bool) {
    let mut upwards_merge: HashMap<PyTile, HashSet<PyTile>> = HashMap::new();
    for tile in merge_set {
        let tile_parent = tile.parent(None);
        let children_set = upwards_merge.entry(tile_parent).or_default();
        children_set.insert(*tile);
    }
    let mut current_tileset: Vec<PyTile> = Vec::new();
    let mut changed = false;
    for (supertile, children) in upwards_merge {
        if children.len() == 4 {
            current_tileset.push(supertile);
            changed = true;
        } else {
            current_tileset.extend(children);
        }
    }
    (current_tileset.into_iter().collect(), changed)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn simplify(args: &Bound<'_, PyTuple>) -> PyResult<HashSet<PyTile>> {
    // Parse tiles from the input sequence
    let tiles_parsed = parse_tiles(args)?;
    let mut tiles = tiles_parsed.into_iter().collect::<Vec<PyTile>>();

    tiles.sort_by_key(|t| t.xyz.z);

    // Check to see if a tile and its parent both already exist.
    // Ensure that tiles are sorted by zoom so parents are encountered first.
    // If so, discard the child (it's covered in the parent)
    let mut root_set: HashSet<PyTile> = HashSet::new();
    for tile in &tiles {
        let mut is_new_tile = true;
        for i in 0..tile.xyz.z {
            let supertile = tile.parent(Some(i));
            if root_set.contains(&supertile) {
                is_new_tile = false;
                break;
            }
        }
        if is_new_tile {
            root_set.insert(*tile);
        }
    }

    // Repeatedly run merge until no further simplification is possible.
    let mut is_merging = true;
    while is_merging {
        let (new_set, changed) = merge(&root_set);
        root_set = new_set;
        is_merging = changed;
    }
    Ok(root_set)
}
