use ndarray::{stack, Array2, Axis};
use std::collections::HashSet;
use utiles_core::{utile, Tile, TileLike};

fn get_range(tiles: &[Tile]) -> (usize, usize, usize, usize) {
    let (xmin, xmax) = tiles.iter().fold((usize::MAX, 0), |(min_x, max_x), tile| {
        (min_x.min(tile.x() as usize), max_x.max(tile.x()))
    });

    let (ymin, ymax) = tiles.iter().fold((usize::MAX, 0), |(min_y, max_y), tile| {
        (min_y.min(tile.y() as usize), max_y.max(tile.y()))
    });

    (xmin, xmax as usize, ymin, ymax as usize)
}

// Function to get zoom level (assumes all tiles have the same zoom).
fn get_zoom(tiles: &[Tile]) -> u8 {
    tiles[0].z
}

// Burn the tiles into a 2D array

fn burn_tiles(
    tiles: &[Tile],
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
) -> Array2<bool> {
    let mut burn = Array2::<bool>::default((xmax - xmin + 3, ymax - ymin + 3));
    for tile in tiles {
        let x_us = tile.x() as usize;
        let y_us = tile.y() as usize;
        burn[(x_us - xmin + 1, y_us - ymin + 1)] = true;
    }
    burn
}

// Roll function for 2D arrays
fn roll_2d(arr: &Array2<bool>, x_shift: isize, y_shift: isize) -> Array2<bool> {
    let (rows, cols) = arr.dim();
    let mut rolled = Array2::default((rows, cols));

    for i in 0..rows {
        for j in 0..cols {
            let new_i = ((i as isize + x_shift).rem_euclid(rows as isize)) as usize;
            let new_j = ((j as isize + y_shift).rem_euclid(cols as isize)) as usize;
            rolled[(new_i, new_j)] = arr[(i, j)];
        }
    }
    rolled
}
static idxs: &'static [(isize, isize)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];
fn find_edges(tiles: Vec<Tile>) -> Vec<Tile> {
    let (xmin, xmax, ymin, ymax) = get_range(&tiles);
    let zoom = get_zoom(&tiles);

    // Create the 2D burn array
    let burn = burn_tiles(&tiles, xmin, xmax, ymin, ymax);

    // Define the rolling indices
    // let idxs = vec![
    //     (-1, -1),
    //     (-1, 0),
    //     (-1, 1),
    //     (0, -1),
    //     (0, 1),
    //     (1, -1),
    //     (1, 0),
    //     (1, 1),
    // ];

    // Create the rolled arrays without adding an extra axis
    let stacks: Vec<Array2<bool>> = idxs
        .into_iter()
        .map(|(dx, dy)| roll_2d(&burn, *dx, *dy))
        .collect();
    // Stack along Axis(2), resulting in a 3D array
    let stacked = stack(
        Axis(2),
        &stacks.iter().map(|a| a.view()).collect::<Vec<_>>(),
    )
    .expect("Failed to stack arrays");

    // Calculate the edges
    // let min_array = stacked.map_axis(Axis(2), |view| view.iter().copied().min().unwrap());
    let min_array = stacked.map_axis(Axis(2), |view| *view.iter().min().unwrap());

    // get edges sans clone (xors the 2 arrs)
    let xys_edge = &burn & !&min_array;

    // Collect the edge tiles
    // let mut edge_indices = Vec::new();
    let uxmin = xmin - 1;
    let uymin = ymin - 1;
    // v1 of weird itering
    // ==========================================
    // let tiles = xys_edge.indexed_iter().map(
    //     |((i, j), is_edge)| {
    //         if *is_edge{
    //             let tile = Tile::new(
    //                 (i + uxmin) as u32,
    //                 (j + uymin) as u32,
    //                 zoom,
    //             );
    //             Some(
    //              tile
    //             )
    //         }else{
    //             None
    //         }
    //
    //     }
    //
    // ).flatten().collect::<Vec<Tile>>();
    // ==========================================
    // more sane version:

    let tiles = xys_edge
        .indexed_iter()
        .filter(|(_, &is_edge)| is_edge)
        .map(|((i, j), _)| Tile::new((i + uxmin) as u32, (j + uymin) as u32, zoom))
        .collect::<Vec<Tile>>();

    // for ((i, j), &is_edge) in xys_edge.indexed_iter() {
    //     if is_edge {
    //         let tile = Tile::new(
    //             (i + uxmin) as u32,
    //             (j + uymin) as u32,
    //             zoom,
    //         );
    //         edge_indices.push(tile);
    //     }
    // }

    tiles
}
// Main function to find edges in the tiles
fn find_edges_cloning(tiles: Vec<Tile>) -> Vec<Tile> {
    let (xmin, xmax, ymin, ymax) = get_range(&tiles);
    let zoom = get_zoom(&tiles);

    // Create the 2D burn array
    let burn = burn_tiles(&tiles, xmin, xmax, ymin, ymax);

    // Define the rolling indices
    // let idxs = vec![
    //     (-1, -1),
    //     (-1, 0),
    //     (-1, 1),
    //     (0, -1),
    //     (0, 1),
    //     (1, -1),
    //     (1, 0),
    //     (1, 1),
    // ];

    // Create the rolled arrays without adding an extra axis
    let mut stacks: Vec<Array2<bool>> = vec![];
    for (dx, dy) in idxs {
        // let rolled =
        // roll_2d(&burn, *dx, *dy);
        stacks.push(roll_2d(&burn, *dx, *dy));
    }

    // Verify shapes of rolled arrays
    for (i, array) in stacks.iter().enumerate() {
        println!("Shape of rolled array {}: {:?}", i, array.shape());
    }

    // Stack along Axis(2), resulting in a shape of [7, 370, 8]
    let stacked = stack(
        Axis(2),
        &stacks.iter().map(|a| a.view()).collect::<Vec<_>>(),
    )
    .expect("Failed to stack arrays");

    println!("Stacked shape: {:?}", stacked.shape());
    println!("Burn shape: {:?}", burn.shape());

    // Calculate the edges
    let min_array =
        stacked.map_axis(Axis(2), |view| view.iter().copied().min().unwrap());
    println!("Shape of min_array: {:?}", min_array.shape());

    let xys_edge = &min_array ^ &burn;

    // set missed to false...
    // previously used:
    // ```
    // let mut xys_edge_bool = xys_edge.clone();
    // for ((i, j), &burn_val) in burn.indexed_iter() {
    //     if !burn_val {
    //         xys_edge_bool[(i, j)] = false;
    //     }
    // }
    // ```
    // but the below is MUCH more sexy
    let mut xys_edge_bool = xys_edge.clone();
    xys_edge_bool &= &burn;

    let mut edge_indices = vec![];
    let uxmin: usize = xmin - 1;
    let uymin: usize = ymin - 1;
    for ((i, j), &is_edge) in xys_edge_bool.indexed_iter() {
        if is_edge {
            let tile = utile!((i + uxmin) as u32, (j + uymin) as u32, zoom);
            // let xyz: (u32, u32, u8) = (
            //     (i + uxmin - 1) as u32,
            //     (j + uymin - 1) as u32,
            //     zoom,
            // );
            edge_indices.push(tile);
        }
    }
    edge_indices
}

fn test_data() -> Vec<Tile> {
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

fn test_expected() -> Vec<Tile> {
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
pub fn edges_main() {
    // let tiles = vec![
    //     Tile { x: 1, y: 1, z: 3 },
    //     Tile { x: 2, y: 1, z: 3 },
    //     Tile { x: 1, y: 2, z: 3 },
    //     Tile { x: 2, y: 2, z: 3 },
    // ];

    let tdata = test_data();
    let edges = find_edges(tdata);

    println!("Edges:\n{:?}", edges);
    let expected = test_expected();

    let expected_set = expected.into_iter().collect::<HashSet<Tile>>();
    let edges_set = edges.into_iter().collect::<HashSet<Tile>>();

    println!("EQUAH: {}", expected_set.eq(&edges_set));
}
