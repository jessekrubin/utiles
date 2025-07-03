// use ahash::HashSet;
use crate::asserts::assert_all_same_zoom;
use crate::{Tile, TileLike, UtilesCoreResult};
#[cfg(feature = "ahash")]
use ahash::HashSet;
#[cfg(not(feature = "ahash"))]
use std::collections::HashSet;

/// Finds edge tiles in a given set of tiles. optionally wraps the x-coordinate
///
/// # Errors
///
/// Errors if the tiles are not all the same zoom level
pub fn find_edges(
    tiles: &[Tile],
    wrapx: bool,
) -> UtilesCoreResult<impl Iterator<Item = Tile> + '_> {
    assert_all_same_zoom(tiles)?;
    let tile_positions: HashSet<(u32, u32)> =
        tiles.iter().map(|tile| (tile.x(), tile.y())).collect();

    let edge_it = tiles.iter().filter_map(move |tile| {
        let is_edge = tile
            .neighbors(wrapx)
            .iter()
            .any(|neighbor| !tile_positions.contains(&(neighbor.x(), neighbor.y())));
        if is_edge { Some(*tile) } else { None }
    });
    Ok(edge_it)
}

// ============================================================================
// previous implementation that uses ndarray
// ============================================================================
// fn burn_tiles(tiles: &[Tile], zbox: TileZBox) -> Array2<bool> {
//     let xmin = zbox.min.x as usize;
//     let ymin = zbox.min.y as usize;
//     // add 3 to pad as dydx ranges are inclusive
//     let dx = zbox.dx() as usize + 3;
//     let dy = zbox.dy() as usize + 3;
//     let mut burn = Array2::<bool>::default((dx, dy));
//     for tile in tiles {
//         let x_us = tile.x() as usize - xmin + 1; // +1 for padding
//         let y_us = tile.y() as usize - ymin + 1; // +1 for padding
//         burn[(x_us, y_us)] = true;
//     }
//     burn
// }

// pub fn find_edges(tiles: &[Tile]) -> UtilesResult<Vec<Tile>> {
//     let zbox = TileZBox::from_tiles(tiles)?;
//     // Create the burn array with padding
//     let burn = burn_tiles(tiles, zbox);
//     let uxmin = (zbox.minx() - 1) as usize; // Adjusted for padding
//     let uymin = (zbox.miny() - 1) as usize; // Adjusted for padding
//
//     let mut edge_tiles = Vec::new();
//     for ((i, j), &is_burn) in burn.indexed_iter() {
//         if is_burn {
//             for &(dx, dy) in IDXS.iter() {
//                 let ni = (i as isize + dx) as usize;
//                 let nj = (j as isize + dy) as usize;
//
//                 // Since we have padding, we don't need to check bounds
//                 if !burn[(ni, nj)] {
//                     // This is an edge tile
//                     let x = (i + uxmin) as u32;
//                     let y = (j + uymin) as u32;
//                     edge_tiles.push(Tile::new(x, y, zbox.zoom));
//                     break;
//                 }
//             }
//         }
//     }
//
//     Ok(edge_tiles)
// }

// ============================================================================
// previous slower implementation
// ============================================================================

// use crate::{UtilesError, UtilesResult};
// use ndarray::{stack, Array2, Axis};
// use utiles_core::{Tile, TileLike, TileZBox};
//
// static IDXS: &[(isize, isize)] = &[
//     (-1, -1),
//     (-1, 0),
//     (-1, 1),
//     (0, -1),
//     (0, 1),
//     (1, -1),
//     (1, 0),
//     (1, 1),
// ];
//
// fn burn_tiles(tiles: &[Tile], zbox: TileZBox) -> Array2<bool> {
//     let xmin = zbox.min.x as usize;
//     let ymin = zbox.min.y as usize;
//     let dx = zbox.dx() as usize;
//     let dy = zbox.dy() as usize;
//     let mut burn = Array2::<bool>::default((dx + 3, dy + 3));
//     for tile in tiles {
//         let x_us = tile.x() as usize;
//         let y_us = tile.y() as usize;
//         burn[(x_us - xmin + 1, y_us - ymin + 1)] = true;
//     }
//     burn
// }
//
// fn roll_2d(arr: &Array2<bool>, x_shift: isize, y_shift: isize) -> Array2<bool> {
//     let (rows, cols) = arr.dim();
//     let mut rolled = Array2::default((rows, cols));
//
//     // rolled.indexed_iter().par_map(
//     // )
//     for i in 0..rows {
//         for j in 0..cols {
//             let new_i = ((i as isize + x_shift).rem_euclid(rows as isize)) as usize;
//             let new_j = ((j as isize + y_shift).rem_euclid(cols as isize)) as usize;
//             rolled[(new_i, new_j)] = arr[(i, j)];
//         }
//     }
//     rolled
// }
//
// #[allow(clippy::similar_names)]
// pub fn find_edges(tiles: &[Tile]) -> UtilesResult<Vec<Tile>> {
//     let zbox = TileZBox::from_tiles(tiles)?;
//     // make 2D burn array
//     let burn = burn_tiles(tiles, zbox);
//
//     // rolled arrays w/o adding an extra axis
//     let stacks: Vec<Array2<bool>> = IDXS
//         .iter()
//         .map(|(dx, dy)| roll_2d(&burn, *dx, *dy))
//         .collect();
//     // stack along axis2, which should be 3d arr
//     let stacked = stack(
//         Axis(2),
//         &stacks.iter().map(|a| a.view()).collect::<Vec<_>>(),
//     )
//     .map_err(UtilesError::NdarrayShapeError)?;
//
//     // edges
//     let min_array =
//         stacked.map_axis(Axis(2), |view| *view.iter().min().unwrap_or(&false));
//     // xor the 2 arrs
//     let xys_edge = &burn & !&min_array;
//
//     // collect the edge tiles
//     let uxmin = (zbox.minx() - 1) as usize;
//     let uymin = (zbox.miny() - 1) as usize;
//
//     // v1 of weird itering
//     // ==========================================
//     // let tiles = xys_edge.indexed_iter().map(
//     //     |((i, j), is_edge)| {
//     //         if *is_edge{
//     //             let tile = Tile::new(
//     //                 (i + uxmin) as u32,
//     //                 (j + uymin) as u32,
//     //                 zoom,
//     //             );
//     //             Some(
//     //              tile
//     //             )
//     //         }else{
//     //             None
//     //         }
//     //
//     //     }
//     //
//     // ).flatten().collect::<Vec<Tile>>();
//     // ==========================================
//     // more sane version:
//
//     let tiles = xys_edge
//         .indexed_iter()
//         .filter(|(_, &is_edge)| is_edge)
//         .map(|((i, j), _)| Tile::new((i + uxmin) as u32, (j + uymin) as u32, zbox.zoom))
//         .collect::<Vec<Tile>>();
//
//     Ok(tiles)
// }
