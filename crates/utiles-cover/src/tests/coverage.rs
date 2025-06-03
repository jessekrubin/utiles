use crate::cover_geojson::geojson2tiles;
use geojson::GeoJson;
use std::collections::HashSet;
use utiles_core::{parse_textiles, Tile};

#[test]
fn cover_geotypes() {
    use crate::cover_geotypes::geometry2tiles;
    let expected = expected_burn_test_tiles();
    let geojson_string = include_str!("./cover-test.geo.json");
    let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();
    let gt_geometry = geo_types::Geometry::<f64>::try_from(geojson.clone()).unwrap();
    let tilescoverage = geometry2tiles(&gt_geometry, 9).unwrap();
    let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
    let expected_set: HashSet<Tile> = expected.into_iter().collect();
    for tile in expected_set {
        assert!(
            tiles_set.contains(&tile),
            "Expected tile {tile:?} is not in the tiles set."
        );
    }
}

#[test]
fn cover_geojson() {
    let expected = expected_burn_test_tiles();
    let geojson_string = include_str!("./cover-test.geo.json");
    let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();
    let tilescoverage = geojson2tiles(&geojson, 9).unwrap();
    let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
    let expected_set: HashSet<Tile> = expected.into_iter().collect();

    for tile in expected_set {
        assert!(
            tiles_set.contains(&tile),
            "Expected tile {tile:?} is not in the tiles set."
        );
    }
}

// macro to test expected vs actual tiles from 2 paths:
// - geojson path file
// - tiles jsonl path file

macro_rules! mk_coverage_test {
    ($test_name:ident, $zoom:expr, $geojson_path:expr, $tiles_path:expr) => {
        #[test]
        fn $test_name() {
            let tiles_str = include_str!($tiles_path);
            let expected: Vec<Tile> = parse_textiles(tiles_str);
            let geojson_string = include_str!($geojson_path);
            let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();
            let gt_geometry =
                geo_types::Geometry::<f64>::try_from(geojson.clone()).unwrap();

            let gt_coverage =
                crate::cover_geotypes::geometry2tiles(&gt_geometry, $zoom).unwrap();
            let gt_tiles_set: HashSet<Tile> = gt_coverage.into_iter().collect();

            let tilescoverage = geojson2tiles(&geojson, $zoom).unwrap();
            let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
            let expected_set: HashSet<Tile> = expected.into_iter().collect();

            println!(
                "Expected tiles: {}, Actual tiles: {}",
                expected_set.len(),
                tiles_set.len()
            );
            println!(
                "GT tiles: {}, Actual GT tiles: {}",
                gt_tiles_set.len(),
                tiles_set.len()
            );
            for tile in expected_set {
                assert!(
                    tiles_set.contains(&tile),
                    "Expected tile {tile:?} is not in the tiles set."
                );

                assert!(
                    gt_tiles_set.contains(&tile),
                    "Expected tile {tile:?} is not in the gt tiles set."
                );
            }
        }
    };
}

mk_coverage_test!(
    cover_geotypes_test,
    9,
    "./cover-test.geo.json",
    "./cover-test.tiles.jsonl"
);
mk_coverage_test!(
    cover_geojson_test,
    9,
    "./cover-test.geo.json",
    "./cover-test.tiles.jsonl"
);

mk_coverage_test!(
    blocky,
    6,
    "../../../../test-data/tile-cover/blocky.geojson",
    "../../../../test-data/tile-cover/blocky.tiles.jsonl"
);

mk_coverage_test!(
    donut,
    16,
    "../../../../test-data/tile-cover/donut.geojson",
    "../../../../test-data/tile-cover/donut.tiles.jsonl"
);

fn expected_burn_test_tiles() -> Vec<Tile> {
    let tiles_str = include_str!("./cover-test.tiles.jsonl");
    parse_textiles(tiles_str)
}
