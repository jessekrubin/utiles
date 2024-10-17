use crate::UtilesResult;
use std::collections::HashSet;
use utiles_core::{Tile, TileLike, TileZBox};

static IDXS: &[(isize, isize)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

static NEIGHBOR_IDXS: &[(i32, i32)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

pub fn find_edges(tiles: &[Tile]) -> UtilesResult<Vec<Tile>> {
    TileZBox::from_tiles(tiles)?;
    // Collect the tiles into a HashSet for O(1) lookup
    let tile_positions: HashSet<(u32, u32)> =
        tiles.iter().map(|tile| (tile.x(), tile.y())).collect();
    let mut edge_tiles = Vec::new();

    for tile in tiles {
        let x = tile.x() as i32;
        let y = tile.y() as i32;

        // if any neighbor is not in the tile_positions das ist uno edge
        let is_edge = NEIGHBOR_IDXS.iter().any(|&(dx, dy)| {
            let neighbor_x = x + dx;
            let neighbor_y = y + dy;

            // TODO: handle wrapping!?
            // if neighbor position is invalid (negative coordinates), consider it as an edge
            if neighbor_x < 0 || neighbor_y < 0 {
                true
            } else {
                let neighbor_pos = (neighbor_x as u32, neighbor_y as u32);
                !tile_positions.contains(&neighbor_pos)
            }
        });

        if is_edge {
            edge_tiles.push(tile.clone());
        }
    }

    Ok(edge_tiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use utiles_core::{utile, Tile};

    fn _test_data_input() -> Vec<Tile> {
        vec![
            utile!(4188, 3104, 13),
            utile!(4192, 2977, 13),
            utile!(4192, 3098, 13),
            utile!(4192, 2983, 13),
            utile!(4192, 2935, 13),
            utile!(4192, 2982, 13),
            utile!(4192, 2980, 13),
            utile!(4192, 3101, 13),
            utile!(4192, 2987, 13),
            utile!(4192, 2987, 13),
            utile!(4192, 2986, 13),
            utile!(4192, 2981, 13),
            utile!(4192, 2997, 13),
            utile!(4192, 2969, 13),
            utile!(4192, 2947, 13),
            utile!(4192, 2927, 13),
            utile!(4192, 2961, 13),
            utile!(4192, 2988, 13),
            utile!(4192, 2976, 13),
            utile!(4192, 2891, 13),
            utile!(4192, 2994, 13),
            utile!(4192, 2959, 13),
            utile!(4192, 2892, 13),
            utile!(4192, 2975, 13),
            utile!(4192, 2931, 13),
            utile!(4192, 2943, 13),
            utile!(4192, 2971, 13),
            utile!(4192, 2931, 13),
            utile!(4192, 2919, 13),
            utile!(4192, 2929, 13),
            utile!(4192, 2930, 13),
            utile!(4192, 2897, 13),
            utile!(4192, 2878, 13),
            utile!(4192, 2879, 13),
            utile!(4192, 2980, 13),
            utile!(4192, 2868, 13),
            utile!(4192, 2887, 13),
            utile!(4192, 2881, 13),
            utile!(4192, 2913, 13),
            utile!(4192, 2884, 13),
            utile!(4192, 2899, 13),
            utile!(4192, 2809, 13),
            utile!(4192, 2859, 13),
            utile!(4192, 2807, 13),
            utile!(4192, 2921, 13),
            utile!(4192, 2775, 13),
            utile!(4192, 2811, 13),
            utile!(4192, 2827, 13),
            utile!(4192, 2867, 13),
            utile!(4192, 2865, 13),
            utile!(4192, 2856, 13),
            utile!(4192, 2873, 13),
            utile!(4192, 2863, 13),
            utile!(4192, 2839, 13),
            utile!(4192, 2774, 13),
            utile!(4192, 2974, 13),
            utile!(4192, 2808, 13),
            utile!(4192, 2832, 13),
            utile!(4192, 2793, 13),
            utile!(4192, 3098, 13),
            utile!(4192, 2787, 13),
            utile!(4192, 2859, 13),
            utile!(4192, 2853, 13),
            utile!(4192, 2825, 13),
            utile!(4192, 2825, 13),
            utile!(4192, 2808, 13),
            utile!(4192, 2787, 13),
            utile!(4192, 2898, 13),
            utile!(4192, 2812, 13),
            utile!(4192, 2859, 13),
            utile!(4192, 2765, 13),
            utile!(4192, 2806, 13),
            utile!(4192, 2769, 13),
            utile!(4192, 2964, 13),
            utile!(4192, 2821, 13),
            utile!(4192, 2778, 13),
            utile!(4192, 2785, 13),
            utile!(4192, 2805, 13),
            utile!(4192, 2737, 13),
            utile!(4192, 2800, 13),
            utile!(4192, 2762, 13),
            utile!(4192, 2756, 13),
            utile!(4192, 2986, 13),
            utile!(4192, 2794, 13),
            utile!(4192, 2760, 13),
            utile!(4192, 2777, 13),
            utile!(4192, 2782, 13),
            utile!(4192, 2746, 13),
            utile!(4192, 2748, 13),
            utile!(4192, 2745, 13),
            utile!(4192, 2871, 13),
            utile!(4192, 2798, 13),
            utile!(4192, 2758, 13),
            utile!(4192, 2756, 13),
            utile!(4192, 2750, 13),
            utile!(4192, 2977, 13),
            utile!(4192, 2765, 13),
            utile!(4192, 2981, 13),
            utile!(4192, 3099, 13),
            utile!(4192, 2983, 13),
        ]
    }

    fn _test_expected() -> Vec<Tile> {
        vec![
            utile!(4188, 3104, 13),
            utile!(4192, 2737, 13),
            utile!(4192, 2745, 13),
            utile!(4192, 2746, 13),
            utile!(4192, 2748, 13),
            utile!(4192, 2750, 13),
            utile!(4192, 2756, 13),
            utile!(4192, 2758, 13),
            utile!(4192, 2760, 13),
            utile!(4192, 2762, 13),
            utile!(4192, 2765, 13),
            utile!(4192, 2769, 13),
            utile!(4192, 2774, 13),
            utile!(4192, 2775, 13),
            utile!(4192, 2777, 13),
            utile!(4192, 2778, 13),
            utile!(4192, 2782, 13),
            utile!(4192, 2785, 13),
            utile!(4192, 2787, 13),
            utile!(4192, 2793, 13),
            utile!(4192, 2794, 13),
            utile!(4192, 2798, 13),
            utile!(4192, 2800, 13),
            utile!(4192, 2805, 13),
            utile!(4192, 2806, 13),
            utile!(4192, 2807, 13),
            utile!(4192, 2808, 13),
            utile!(4192, 2809, 13),
            utile!(4192, 2811, 13),
            utile!(4192, 2812, 13),
            utile!(4192, 2821, 13),
            utile!(4192, 2825, 13),
            utile!(4192, 2827, 13),
            utile!(4192, 2832, 13),
            utile!(4192, 2839, 13),
            utile!(4192, 2853, 13),
            utile!(4192, 2856, 13),
            utile!(4192, 2859, 13),
            utile!(4192, 2863, 13),
            utile!(4192, 2865, 13),
            utile!(4192, 2867, 13),
            utile!(4192, 2868, 13),
            utile!(4192, 2871, 13),
            utile!(4192, 2873, 13),
            utile!(4192, 2878, 13),
            utile!(4192, 2879, 13),
            utile!(4192, 2881, 13),
            utile!(4192, 2884, 13),
            utile!(4192, 2887, 13),
            utile!(4192, 2891, 13),
            utile!(4192, 2892, 13),
            utile!(4192, 2897, 13),
            utile!(4192, 2898, 13),
            utile!(4192, 2899, 13),
            utile!(4192, 2913, 13),
            utile!(4192, 2919, 13),
            utile!(4192, 2921, 13),
            utile!(4192, 2927, 13),
            utile!(4192, 2929, 13),
            utile!(4192, 2930, 13),
            utile!(4192, 2931, 13),
            utile!(4192, 2935, 13),
            utile!(4192, 2943, 13),
            utile!(4192, 2947, 13),
            utile!(4192, 2959, 13),
            utile!(4192, 2961, 13),
            utile!(4192, 2964, 13),
            utile!(4192, 2969, 13),
            utile!(4192, 2971, 13),
            utile!(4192, 2974, 13),
            utile!(4192, 2975, 13),
            utile!(4192, 2976, 13),
            utile!(4192, 2977, 13),
            utile!(4192, 2980, 13),
            utile!(4192, 2981, 13),
            utile!(4192, 2982, 13),
            utile!(4192, 2983, 13),
            utile!(4192, 2986, 13),
            utile!(4192, 2987, 13),
            utile!(4192, 2988, 13),
            utile!(4192, 2994, 13),
            utile!(4192, 2997, 13),
            utile!(4192, 3098, 13),
            utile!(4192, 3099, 13),
            utile!(4192, 3101, 13),
        ]
    }
    #[test]
    fn test_edges() {
        let tdata = _test_data_input();
        let edges = find_edges(&tdata).unwrap();
        let expected = _test_expected();
        let expected_set = expected.into_iter().collect::<HashSet<Tile>>();
        let edges_set = edges.into_iter().collect::<HashSet<Tile>>();

        assert_eq!(expected_set, edges_set);
    }
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
