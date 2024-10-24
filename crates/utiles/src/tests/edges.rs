#![allow(clippy::too_many_lines)]

use crate::edges::find_edges;
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

    let mut edges = vec![];
    for t in find_edges(&tdata).unwrap() {
        edges.push(t);
    }
    // let edges = find_edges_vec(&tdata).unwrap();
    let expected = _test_expected();
    let expected_set = expected.into_iter().collect::<HashSet<Tile>>();
    let edges_set = edges.into_iter().collect::<HashSet<Tile>>();

    assert_eq!(expected_set, edges_set);
}
