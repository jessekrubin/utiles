use crate::cover::{geojson2tiles};
use geojson::GeoJson;
use std::collections::HashSet;
use utiles_core::{parse_textiles, Tile};

//
// #[test]
// fn cover_geotype() {
//     let expected = expected_burn_test_tiles();
//     let geojson_string = r#"
//     {
//     "type":"FeatureCollection",
//     "features":[
//     {"type":"Feature",
//     "properties":{},
//     "geometry":{"type":"Polygon","coordinates":[[[-120.76171875,38.788345355085625],[-113.15917968749999,42.87596410238254],[-114.9609375,43.03677585761058],[-116.806640625,42.35854391749705],[-122.78320312499999,45.1510532655634],[-120.41015624999999,44.33956524809713],[-115.927734375,45.98169518512228],[-115.7080078125,44.9336963896947],[-110.61035156249999,45.460130637921004],[-113.64257812499999,46.37725420510028],[-109.9951171875,47.69497434186282],[-109.1162109375,46.28622391806708],[-103.9306640625,47.040182144806664],[-106.435546875,44.49650533109345],[-107.1826171875,45.42929873257377],[-105.9521484375,45.9511496866914],[-108.9404296875,45.706179285330855],[-105.8203125,41.80407814427237],[-108.10546875,41.343824581185686],[-110.0830078125,43.992814500489914],[-110.74218749999999,40.979898069620155],[-111.62109375,41.705728515237524],[-111.97265625,39.13006024213511],[-105.29296874999999,38.92522904714054],[-103.71093749999999,42.293564192170095],[-104.765625,43.389081939117496],[-103.22753906249999,45.644768217751924],[-100.986328125,42.261049162113856],[-103.798828125,36.914764288955936],[-112.412109375,36.06686213257888],[-115.00488281250001,38.06539235133249],[-113.37890625,41.44272637767212],[-119.091796875,37.54457732085582],[-123.96972656249999,38.61687046392973],[-121.728515625,41.07935114946899],[-126.21093749999999,43.13306116240612],[-127.3095703125,45.460130637921004],[-124.27734374999999,47.81315451752768],[-118.037109375,47.81315451752768],[-124.01367187499999,45.85941212790755],[-124.67285156250001,43.929549935614595],[-118.740234375,41.37680856570233],
//     [-120.76171875,38.788345355085625]]]}}]}
//     "#;
//
//     let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();
//     let geom = geo_types::Geometry::<f64>::try_from(geojson).unwrap();
//
//     println!("Geometry: {geom:?}");
//     let tilescoverage = geometry2tiles(&geom, 9, None).unwrap();
//     let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
//     let expected_set: HashSet<Tile> = expected.into_iter().collect();
//     for tile in expected_set {
//         assert!(
//             tiles_set.contains(&tile),
//             "Expected tile {tile:?} is not in the tiles set."
//         );
//     }
//
// }
#[test]
fn cover_geotypes() {
    use crate:: cover_geotypes::geometry2tiles;
    let expected = expected_burn_test_tiles();
    let geojson_string = include_str!("./cover-test.geo.json");
    // let geojson_string = r#"
    // {
    // "type":"FeatureCollection",
    // "features":[
    // {"type":"Feature",
    // "properties":{},
    // "geometry":{"type":"Polygon","coordinates":[[[-120.76171875,38.788345355085625],[-113.15917968749999,42.87596410238254],[-114.9609375,43.03677585761058],[-116.806640625,42.35854391749705],[-122.78320312499999,45.1510532655634],[-120.41015624999999,44.33956524809713],[-115.927734375,45.98169518512228],[-115.7080078125,44.9336963896947],[-110.61035156249999,45.460130637921004],[-113.64257812499999,46.37725420510028],[-109.9951171875,47.69497434186282],[-109.1162109375,46.28622391806708],[-103.9306640625,47.040182144806664],[-106.435546875,44.49650533109345],[-107.1826171875,45.42929873257377],[-105.9521484375,45.9511496866914],[-108.9404296875,45.706179285330855],[-105.8203125,41.80407814427237],[-108.10546875,41.343824581185686],[-110.0830078125,43.992814500489914],[-110.74218749999999,40.979898069620155],[-111.62109375,41.705728515237524],[-111.97265625,39.13006024213511],[-105.29296874999999,38.92522904714054],[-103.71093749999999,42.293564192170095],[-104.765625,43.389081939117496],[-103.22753906249999,45.644768217751924],[-100.986328125,42.261049162113856],[-103.798828125,36.914764288955936],[-112.412109375,36.06686213257888],[-115.00488281250001,38.06539235133249],[-113.37890625,41.44272637767212],[-119.091796875,37.54457732085582],[-123.96972656249999,38.61687046392973],[-121.728515625,41.07935114946899],[-126.21093749999999,43.13306116240612],[-127.3095703125,45.460130637921004],[-124.27734374999999,47.81315451752768],[-118.037109375,47.81315451752768],[-124.01367187499999,45.85941212790755],[-124.67285156250001,43.929549935614595],[-118.740234375,41.37680856570233],
    // [-120.76171875,38.788345355085625]]]}}]}
    // "#;

    let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();
    let gt_geometry = geo_types::Geometry::<f64>::try_from(geojson.clone()).unwrap();
    let tilescoverage = geometry2tiles(&gt_geometry, 9, None).unwrap();
    let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
    let expected_set: HashSet<Tile> = expected.into_iter().collect();
    for tile in expected_set {
        assert!(
            tiles_set.contains(&tile),
            "Expected tile {tile:?} is not in the tiles set."
        );
    }

    // // Find common elements (intersection)
    // let common: HashSet<_> = tiles_set.intersection(&expected_set).copied().collect();
    // assert!(
    //     !common.is_empty(),
    //     "No common elements between tiles and expected tiles."
    // );

    // // Find elements only in expected_set
    // let expected_only: HashSet<_> =
    //     expected_set.difference(&tiles_set).copied().collect();
    // assert!(
    //     expected_only.is_empty(),
    //     "Expected set contains additional tiles: {expected_only:?}"
    // );

    // // Find elements only in tiles_set
    // let tiles_only: HashSet<_> = tiles_set.difference(&expected_set).copied().collect();
    // assert!(
    //     tiles_only.is_empty(),
    //     "Tiles set contains additional tiles: {tiles_only:?}"
    // );
}


#[test]
fn cover_geojson() {
    let expected = expected_burn_test_tiles();
    let geojson_string = include_str!("./cover-test.geo.json");
    // let geojson_string = r#"
    // {
    // "type":"FeatureCollection",
    // "features":[
    // {"type":"Feature",
    // "properties":{},
    // "geometry":{"type":"Polygon","coordinates":[[[-120.76171875,38.788345355085625],[-113.15917968749999,42.87596410238254],[-114.9609375,43.03677585761058],[-116.806640625,42.35854391749705],[-122.78320312499999,45.1510532655634],[-120.41015624999999,44.33956524809713],[-115.927734375,45.98169518512228],[-115.7080078125,44.9336963896947],[-110.61035156249999,45.460130637921004],[-113.64257812499999,46.37725420510028],[-109.9951171875,47.69497434186282],[-109.1162109375,46.28622391806708],[-103.9306640625,47.040182144806664],[-106.435546875,44.49650533109345],[-107.1826171875,45.42929873257377],[-105.9521484375,45.9511496866914],[-108.9404296875,45.706179285330855],[-105.8203125,41.80407814427237],[-108.10546875,41.343824581185686],[-110.0830078125,43.992814500489914],[-110.74218749999999,40.979898069620155],[-111.62109375,41.705728515237524],[-111.97265625,39.13006024213511],[-105.29296874999999,38.92522904714054],[-103.71093749999999,42.293564192170095],[-104.765625,43.389081939117496],[-103.22753906249999,45.644768217751924],[-100.986328125,42.261049162113856],[-103.798828125,36.914764288955936],[-112.412109375,36.06686213257888],[-115.00488281250001,38.06539235133249],[-113.37890625,41.44272637767212],[-119.091796875,37.54457732085582],[-123.96972656249999,38.61687046392973],[-121.728515625,41.07935114946899],[-126.21093749999999,43.13306116240612],[-127.3095703125,45.460130637921004],[-124.27734374999999,47.81315451752768],[-118.037109375,47.81315451752768],[-124.01367187499999,45.85941212790755],[-124.67285156250001,43.929549935614595],[-118.740234375,41.37680856570233],
    // [-120.76171875,38.788345355085625]]]}}]}
    // "#;

    let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();

    let tilescoverage = geojson2tiles(&geojson, 9, None).unwrap();

    let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();
    let expected_set: HashSet<Tile> = expected.into_iter().collect();

    for tile in expected_set {
        assert!(
            tiles_set.contains(&tile),
            "Expected tile {tile:?} is not in the tiles set."
        );
    }

    // // Find common elements (intersection)
    // let common: HashSet<_> = tiles_set.intersection(&expected_set).copied().collect();
    // assert!(
    //     !common.is_empty(),
    //     "No common elements between tiles and expected tiles."
    // );

    // // Find elements only in expected_set
    // let expected_only: HashSet<_> =
    //     expected_set.difference(&tiles_set).copied().collect();
    // assert!(
    //     expected_only.is_empty(),
    //     "Expected set contains additional tiles: {expected_only:?}"
    // );

    // // Find elements only in tiles_set
    // let tiles_only: HashSet<_> = tiles_set.difference(&expected_set).copied().collect();
    // assert!(
    //     tiles_only.is_empty(),
    //     "Tiles set contains additional tiles: {tiles_only:?}"
    // );
}

#[allow(clippy::too_many_lines)]
fn expected_burn_test_tiles() -> Vec<Tile> {
    let tiles_str = include_str!("./cover-test.tiles.jsonl");
    parse_textiles(tiles_str)
}
