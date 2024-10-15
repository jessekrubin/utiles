use geojson::GeoJson;
use std::collections::HashSet;
use utiles::cover::geojson2tiles;
use utiles_core::{lnglat2tile_frac, parse_textiles, tile, utile, Tile};

fn expected_burn_test_tiles() -> Vec<Tile> {
    let tiles_str = r#"[78, 178, 9]
[79, 178, 9]
[80, 178, 9]
[81, 178, 9]
[82, 178, 9]
[83, 178, 9]
[84, 178, 9]
[85, 178, 9]
[86, 178, 9]
[87, 178, 9]
[88, 178, 9]
[98, 178, 9]
[99, 178, 9]
[77, 179, 9]
[78, 179, 9]
[79, 179, 9]
[80, 179, 9]
[81, 179, 9]
[82, 179, 9]
[83, 179, 9]
[84, 179, 9]
[85, 179, 9]
[86, 179, 9]
[96, 179, 9]
[97, 179, 9]
[98, 179, 9]
[99, 179, 9]
[100, 179, 9]
[76, 180, 9]
[77, 180, 9]
[78, 180, 9]
[79, 180, 9]
[80, 180, 9]
[81, 180, 9]
[82, 180, 9]
[83, 180, 9]
[84, 180, 9]
[95, 180, 9]
[96, 180, 9]
[97, 180, 9]
[98, 180, 9]
[99, 180, 9]
[100, 180, 9]
[103, 180, 9]
[104, 180, 9]
[105, 180, 9]
[106, 180, 9]
[107, 180, 9]
[108, 180, 9]
[76, 181, 9]
[77, 181, 9]
[78, 181, 9]
[79, 181, 9]
[80, 181, 9]
[81, 181, 9]
[82, 181, 9]
[94, 181, 9]
[95, 181, 9]
[96, 181, 9]
[97, 181, 9]
[98, 181, 9]
[99, 181, 9]
[100, 181, 9]
[101, 181, 9]
[102, 181, 9]
[103, 181, 9]
[104, 181, 9]
[105, 181, 9]
[106, 181, 9]
[107, 181, 9]
[75, 182, 9]
[76, 182, 9]
[77, 182, 9]
[78, 182, 9]
[79, 182, 9]
[80, 182, 9]
[89, 182, 9]
[90, 182, 9]
[91, 182, 9]
[95, 182, 9]
[96, 182, 9]
[97, 182, 9]
[98, 182, 9]
[99, 182, 9]
[100, 182, 9]
[101, 182, 9]
[102, 182, 9]
[103, 182, 9]
[104, 182, 9]
[105, 182, 9]
[106, 182, 9]
[109, 182, 9]
[74, 183, 9]
[75, 183, 9]
[76, 183, 9]
[77, 183, 9]
[78, 183, 9]
[79, 183, 9]
[81, 183, 9]
[87, 183, 9]
[88, 183, 9]
[89, 183, 9]
[90, 183, 9]
[91, 183, 9]
[93, 183, 9]
[94, 183, 9]
[95, 183, 9]
[96, 183, 9]
[97, 183, 9]
[98, 183, 9]
[99, 183, 9]
[100, 183, 9]
[101, 183, 9]
[103, 183, 9]
[104, 183, 9]
[105, 183, 9]
[106, 183, 9]
[108, 183, 9]
[109, 183, 9]
[75, 184, 9]
[76, 184, 9]
[77, 184, 9]
[78, 184, 9]
[79, 184, 9]
[81, 184, 9]
[82, 184, 9]
[83, 184, 9]
[85, 184, 9]
[86, 184, 9]
[87, 184, 9]
[88, 184, 9]
[89, 184, 9]
[90, 184, 9]
[91, 184, 9]
[92, 184, 9]
[93, 184, 9]
[94, 184, 9]
[95, 184, 9]
[96, 184, 9]
[97, 184, 9]
[98, 184, 9]
[99, 184, 9]
[100, 184, 9]
[101, 184, 9]
[102, 184, 9]
[103, 184, 9]
[104, 184, 9]
[105, 184, 9]
[108, 184, 9]
[109, 184, 9]
[110, 184, 9]
[75, 185, 9]
[76, 185, 9]
[77, 185, 9]
[78, 185, 9]
[79, 185, 9]
[83, 185, 9]
[84, 185, 9]
[85, 185, 9]
[86, 185, 9]
[87, 185, 9]
[88, 185, 9]
[89, 185, 9]
[90, 185, 9]
[91, 185, 9]
[92, 185, 9]
[93, 185, 9]
[94, 185, 9]
[95, 185, 9]
[96, 185, 9]
[97, 185, 9]
[98, 185, 9]
[99, 185, 9]
[100, 185, 9]
[101, 185, 9]
[102, 185, 9]
[104, 185, 9]
[107, 185, 9]
[108, 185, 9]
[109, 185, 9]
[110, 185, 9]
[75, 186, 9]
[76, 186, 9]
[77, 186, 9]
[78, 186, 9]
[79, 186, 9]
[84, 186, 9]
[85, 186, 9]
[86, 186, 9]
[87, 186, 9]
[88, 186, 9]
[89, 186, 9]
[90, 186, 9]
[91, 186, 9]
[92, 186, 9]
[93, 186, 9]
[94, 186, 9]
[95, 186, 9]
[96, 186, 9]
[97, 186, 9]
[98, 186, 9]
[99, 186, 9]
[100, 186, 9]
[101, 186, 9]
[102, 186, 9]
[103, 186, 9]
[107, 186, 9]
[108, 186, 9]
[109, 186, 9]
[110, 186, 9]
[111, 186, 9]
[76, 187, 9]
[77, 187, 9]
[78, 187, 9]
[79, 187, 9]
[80, 187, 9]
[81, 187, 9]
[86, 187, 9]
[87, 187, 9]
[88, 187, 9]
[89, 187, 9]
[90, 187, 9]
[91, 187, 9]
[92, 187, 9]
[93, 187, 9]
[94, 187, 9]
[95, 187, 9]
[96, 187, 9]
[97, 187, 9]
[98, 187, 9]
[99, 187, 9]
[100, 187, 9]
[101, 187, 9]
[102, 187, 9]
[103, 187, 9]
[104, 187, 9]
[107, 187, 9]
[108, 187, 9]
[109, 187, 9]
[110, 187, 9]
[111, 187, 9]
[76, 188, 9]
[77, 188, 9]
[78, 188, 9]
[79, 188, 9]
[80, 188, 9]
[81, 188, 9]
[82, 188, 9]
[83, 188, 9]
[87, 188, 9]
[88, 188, 9]
[89, 188, 9]
[90, 188, 9]
[91, 188, 9]
[92, 188, 9]
[93, 188, 9]
[94, 188, 9]
[95, 188, 9]
[96, 188, 9]
[97, 188, 9]
[98, 188, 9]
[99, 188, 9]
[100, 188, 9]
[101, 188, 9]
[102, 188, 9]
[103, 188, 9]
[104, 188, 9]
[107, 188, 9]
[108, 188, 9]
[109, 188, 9]
[110, 188, 9]
[111, 188, 9]
[112, 188, 9]
[78, 189, 9]
[79, 189, 9]
[80, 189, 9]
[81, 189, 9]
[82, 189, 9]
[83, 189, 9]
[84, 189, 9]
[89, 189, 9]
[90, 189, 9]
[92, 189, 9]
[93, 189, 9]
[94, 189, 9]
[95, 189, 9]
[96, 189, 9]
[97, 189, 9]
[98, 189, 9]
[100, 189, 9]
[101, 189, 9]
[102, 189, 9]
[103, 189, 9]
[104, 189, 9]
[105, 189, 9]
[108, 189, 9]
[109, 189, 9]
[110, 189, 9]
[111, 189, 9]
[112, 189, 9]
[79, 190, 9]
[80, 190, 9]
[81, 190, 9]
[82, 190, 9]
[83, 190, 9]
[84, 190, 9]
[85, 190, 9]
[86, 190, 9]
[91, 190, 9]
[92, 190, 9]
[93, 190, 9]
[94, 190, 9]
[95, 190, 9]
[96, 190, 9]
[97, 190, 9]
[98, 190, 9]
[101, 190, 9]
[102, 190, 9]
[103, 190, 9]
[104, 190, 9]
[105, 190, 9]
[107, 190, 9]
[108, 190, 9]
[109, 190, 9]
[110, 190, 9]
[111, 190, 9]
[112, 190, 9]
[81, 191, 9]
[82, 191, 9]
[83, 191, 9]
[84, 191, 9]
[85, 191, 9]
[86, 191, 9]
[87, 191, 9]
[89, 191, 9]
[90, 191, 9]
[91, 191, 9]
[92, 191, 9]
[93, 191, 9]
[94, 191, 9]
[95, 191, 9]
[96, 191, 9]
[97, 191, 9]
[98, 191, 9]
[102, 191, 9]
[103, 191, 9]
[107, 191, 9]
[108, 191, 9]
[109, 191, 9]
[110, 191, 9]
[111, 191, 9]
[82, 192, 9]
[83, 192, 9]
[84, 192, 9]
[85, 192, 9]
[86, 192, 9]
[88, 192, 9]
[89, 192, 9]
[90, 192, 9]
[91, 192, 9]
[92, 192, 9]
[93, 192, 9]
[94, 192, 9]
[95, 192, 9]
[96, 192, 9]
[97, 192, 9]
[107, 192, 9]
[108, 192, 9]
[109, 192, 9]
[110, 192, 9]
[111, 192, 9]
[81, 193, 9]
[82, 193, 9]
[83, 193, 9]
[84, 193, 9]
[85, 193, 9]
[86, 193, 9]
[87, 193, 9]
[88, 193, 9]
[89, 193, 9]
[90, 193, 9]
[91, 193, 9]
[92, 193, 9]
[93, 193, 9]
[94, 193, 9]
[95, 193, 9]
[96, 193, 9]
[97, 193, 9]
[106, 193, 9]
[107, 193, 9]
[108, 193, 9]
[109, 193, 9]
[110, 193, 9]
[80, 194, 9]
[81, 194, 9]
[82, 194, 9]
[83, 194, 9]
[84, 194, 9]
[85, 194, 9]
[86, 194, 9]
[87, 194, 9]
[88, 194, 9]
[89, 194, 9]
[90, 194, 9]
[91, 194, 9]
[93, 194, 9]
[94, 194, 9]
[95, 194, 9]
[96, 194, 9]
[106, 194, 9]
[107, 194, 9]
[108, 194, 9]
[109, 194, 9]
[110, 194, 9]
[79, 195, 9]
[80, 195, 9]
[81, 195, 9]
[82, 195, 9]
[83, 195, 9]
[84, 195, 9]
[85, 195, 9]
[86, 195, 9]
[87, 195, 9]
[88, 195, 9]
[89, 195, 9]
[90, 195, 9]
[92, 195, 9]
[93, 195, 9]
[94, 195, 9]
[95, 195, 9]
[96, 195, 9]
[97, 195, 9]
[98, 195, 9]
[99, 195, 9]
[100, 195, 9]
[101, 195, 9]
[102, 195, 9]
[103, 195, 9]
[104, 195, 9]
[105, 195, 9]
[106, 195, 9]
[107, 195, 9]
[108, 195, 9]
[109, 195, 9]
[110, 195, 9]
[79, 196, 9]
[80, 196, 9]
[81, 196, 9]
[82, 196, 9]
[83, 196, 9]
[84, 196, 9]
[85, 196, 9]
[86, 196, 9]
[87, 196, 9]
[88, 196, 9]
[89, 196, 9]
[92, 196, 9]
[93, 196, 9]
[94, 196, 9]
[95, 196, 9]
[96, 196, 9]
[97, 196, 9]
[98, 196, 9]
[99, 196, 9]
[100, 196, 9]
[101, 196, 9]
[102, 196, 9]
[103, 196, 9]
[104, 196, 9]
[105, 196, 9]
[106, 196, 9]
[107, 196, 9]
[108, 196, 9]
[109, 196, 9]
[81, 197, 9]
[82, 197, 9]
[83, 197, 9]
[84, 197, 9]
[85, 197, 9]
[86, 197, 9]
[87, 197, 9]
[88, 197, 9]
[92, 197, 9]
[93, 197, 9]
[94, 197, 9]
[95, 197, 9]
[96, 197, 9]
[97, 197, 9]
[98, 197, 9]
[99, 197, 9]
[100, 197, 9]
[101, 197, 9]
[102, 197, 9]
[103, 197, 9]
[104, 197, 9]
[105, 197, 9]
[106, 197, 9]
[107, 197, 9]
[108, 197, 9]
[109, 197, 9]
[85, 198, 9]
[86, 198, 9]
[93, 198, 9]
[94, 198, 9]
[95, 198, 9]
[96, 198, 9]
[97, 198, 9]
[98, 198, 9]
[99, 198, 9]
[100, 198, 9]
[101, 198, 9]
[102, 198, 9]
[103, 198, 9]
[104, 198, 9]
[105, 198, 9]
[106, 198, 9]
[107, 198, 9]
[108, 198, 9]
[94, 199, 9]
[95, 199, 9]
[96, 199, 9]
[97, 199, 9]
[98, 199, 9]
[99, 199, 9]
[100, 199, 9]
[101, 199, 9]
[102, 199, 9]
[103, 199, 9]
[104, 199, 9]
[105, 199, 9]
[106, 199, 9]
[107, 199, 9]
[108, 199, 9]
[95, 200, 9]
[96, 200, 9]
[97, 200, 9]
[98, 200, 9]
[99, 200, 9]
[100, 200, 9]
[101, 200, 9]
[102, 200, 9]
[103, 200, 9]"#;
    parse_textiles(tiles_str)
}
// fn to_id(x: u32, y: u32, z: u8) -> u64 {
//     ((2u64 * (1u64 << z)) * y as u64 + x as u64) * 32u64 + z as u64
// }
// fn from_id(id: u64) -> Tile {
//     let z = (id % 32) as u8;
//     let dim = 2u64 * (1u64 << z);
//     let xy = (id - z as u64) / 32u64;
//     let x = (xy % dim) as u32;
//     let y = ((xy - x as u64) / dim) as u32;
//     utile!(x, y, z)
// }
// fn line_string_cover(
//     tile_hash: &mut HashSet<u64>,
//     coords: &[(f64, f64)],
//     maxzoom: u8,
//     mut ring: Option<&mut Vec<(u32, u32)>>,
// ) {
//     let mut prev_x: Option<u32> = None;
//     let mut prev_y: Option<u32> = None;
//
//     let n = 1u32 << maxzoom; // Number of tiles at this zoom level
//
//     for i in 0..coords.len() - 1 {
//         let start_coord = coords[i];
//         let stop_coord = coords[i + 1];
//
//         let (x0f, y0f, _) = lnglat2tile_frac(start_coord.0, start_coord.1, maxzoom);
//         let (x1f, y1f, _) = lnglat2tile_frac(stop_coord.0, stop_coord.1, maxzoom);
//
//         let dx = x1f - x0f;
//         let dy = y1f - y0f;
//
//         if dx == 0.0 && dy == 0.0 {
//             continue;
//         }
//
//         let sx = dx.signum();
//         let sy = dy.signum();
//
//         let mut x = x0f.floor() as i64;
//         let mut y = y0f.floor() as i64;
//
//         let tdx = if dx != 0.0 {
//             (sx / dx).abs()
//         } else {
//             f64::INFINITY
//         };
//         let tdy = if dy != 0.0 {
//             (sy / dy).abs()
//         } else {
//             f64::INFINITY
//         };
//
//         let mut t_max_x = if dx == 0.0 {
//             f64::INFINITY
//         } else {
//             ((if dx > 0.0 { 1.0 } else { 0.0 } + x as f64 - x0f) / dx).abs()
//         };
//         let mut t_max_y = if dy == 0.0 {
//             f64::INFINITY
//         } else {
//             ((if dy > 0.0 { 1.0 } else { 0.0 } + y as f64 - y0f) / dy).abs()
//         };
//
//         // Wrap x and clamp y to valid ranges
//         x = x.rem_euclid(n as i64);
//         y = y.clamp(0, (n - 1) as i64);
//
//         let x_u32 = x as u32;
//         let y_u32 = y as u32;
//
//         if prev_x.is_none()
//             || prev_y.is_none()
//             || x_u32 != prev_x.unwrap()
//             || y_u32 != prev_y.unwrap()
//         {
//             tile_hash.insert(to_id(x_u32, y_u32, maxzoom));
//             if let Some(ring) = &mut ring {
//                 if prev_y != Some(y_u32) {
//                     ring.push((x_u32, y_u32));
//                 }
//             }
//             prev_x = Some(x_u32);
//             prev_y = Some(y_u32);
//         }
//
//         while t_max_x < 1.0 || t_max_y < 1.0 {
//             if t_max_x < t_max_y {
//                 t_max_x += tdx;
//                 x += sx as i64;
//             } else {
//                 t_max_y += tdy;
//                 y += sy as i64;
//             }
//
//             // Wrap x and clamp y
//             x = x.rem_euclid(n as i64);
//             y = y.clamp(0, (n - 1) as i64);
//
//             let x_u32 = x as u32;
//             let y_u32 = y as u32;
//
//             tile_hash.insert(to_id(x_u32, y_u32, maxzoom));
//             if let Some(ring) = &mut ring {
//                 if prev_y != Some(y_u32) {
//                     ring.push((x_u32, y_u32));
//                 }
//             }
//             prev_x = Some(x_u32);
//             prev_y = Some(y_u32);
//         }
//     }
// }

// fn polygon_cover(
//     tile_hash: &mut HashSet<u64>,
//     tile_array: &mut Vec<Tile>,
//     geom: &[Vec<(f64, f64)>],
//     zoom: u8,
// ) {
//     let mut intersections = Vec::new();
//     for element in geom {
//         let mut ring = Vec::new();
//         line_string_cover(tile_hash, element, zoom, Some(&mut ring));
//         let len = ring.len();
//         for j in 0..len {
//             let k = if j == 0 { len - 1 } else { j - 1 };
//             let m = (j + 1) % len;
//
//             let ring_j = ring[j];
//             let ring_k = ring[k];
//             let ring_m = ring[m];
//
//             let y = ring_j.1;
//
//             if (y > ring_k.1 || y > ring_m.1) // Not local minimum
//                 && (y < ring_k.1 || y < ring_m.1) // Not local maximum
//                 && y != ring_m.1
//             {
//                 intersections.push(ring_j);
//             }
//         }
//     }
//     intersections.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
//     let mut i = 0;
//     while i < intersections.len() {
//         let y = intersections[i].1;
//         let min_x = intersections[i].0 + 1;
//         if i + 1 >= intersections.len() {
//             break;
//         }
//         let max_x = intersections[i + 1].0;
//         for x in min_x..max_x {
//             tile_array.push(utile!(x, y, zoom));
//         }
//         i += 2;
//     }
// }
// fn geom2tiles(geom: &geojson::Geometry, zoom: u8) -> Vec<Tile> {
//     let mut tile_hash = HashSet::new();
//     let mut tiles = Vec::new();
//
//     match &geom.value {
//         geojson::Value::Point(coords) => {
//             let tile = tile(coords[0], coords[1], zoom, None).unwrap();
//             tiles.push(tile);
//             return tiles;
//         }
//         geojson::Value::MultiPoint(coords_list) => {
//             for coords in coords_list {
//                 let tile = tile(coords[0], coords[1], zoom, None).unwrap();
//                 tile_hash.insert(to_id(tile.x, tile.y, tile.z));
//             }
//         }
//         geojson::Value::LineString(coords_list) => {
//             let coords: Vec<(f64, f64)> =
//                 coords_list.iter().map(|c| (c[0], c[1])).collect();
//             line_string_cover(&mut tile_hash, &coords, zoom, None);
//         }
//         geojson::Value::MultiLineString(coords_lists) => {
//             for coords_list in coords_lists {
//                 let coords: Vec<(f64, f64)> =
//                     coords_list.iter().map(|c| (c[0], c[1])).collect();
//                 line_string_cover(&mut tile_hash, &coords, zoom, None);
//             }
//         }
//         geojson::Value::Polygon(coords_lists) => {
//             let coords: Vec<Vec<(f64, f64)>> = coords_lists
//                 .iter()
//                 .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
//                 .collect();
//             polygon_cover(&mut tile_hash, &mut tiles, &coords, zoom);
//         }
//         geojson::Value::MultiPolygon(coords_list_of_lists) => {
//             for coords_lists in coords_list_of_lists {
//                 let coords: Vec<Vec<(f64, f64)>> = coords_lists
//                     .iter()
//                     .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
//                     .collect();
//                 polygon_cover(&mut tile_hash, &mut tiles, &coords, zoom);
//             }
//         }
//         _ => {
//             panic!("Geometry type not implemented");
//         }
//     }
//     append_hash_tiles(&tile_hash, &mut tiles);
//     tiles
// }
// fn append_hash_tiles(tile_hash: &HashSet<u64>, tiles: &mut Vec<Tile>) {
//     for id in tile_hash {
//         tiles.push(from_id(*id));
//     }
// }
//
// fn geojson2tiles(gj: GeoJson, zoom: u8) -> HashSet<Tile> {
//     let mut tilescoverage: HashSet<Tile> = HashSet::new();
//
//     match gj {
//         GeoJson::FeatureCollection(ref ctn) => {
//             for feature in &ctn.features {
//                 if let Some(ref geom) = feature.geometry {
//                     let cov = geom2tiles(geom, zoom);
//                     tilescoverage.extend(cov);
//                 }
//             }
//         }
//         GeoJson::Feature(ref feature) => {
//             if let Some(ref geom) = feature.geometry {
//                 // match_geometry(geom)
//                 let cov = geom2tiles(geom, zoom);
//                 tilescoverage.extend(cov);
//             }
//         }
//         GeoJson::Geometry(ref geom) => {
//             let cov = geom2tiles(geom, zoom);
//             tilescoverage.extend(cov);
//         }
//     }
//     tilescoverage
// }
pub fn burn_main() {
    println!("-----------------------------");

    let expected = expected_burn_test_tiles();
    println!("expected tiles: {:?}", expected);
    let geojson_string = r#"
    {
    "type":"FeatureCollection",
    "features":[
    {"type":"Feature",
    "properties":{},
    "geometry":{"type":"Polygon","coordinates":[[[-120.76171875,38.788345355085625],[-113.15917968749999,42.87596410238254],[-114.9609375,43.03677585761058],[-116.806640625,42.35854391749705],[-122.78320312499999,45.1510532655634],[-120.41015624999999,44.33956524809713],[-115.927734375,45.98169518512228],[-115.7080078125,44.9336963896947],[-110.61035156249999,45.460130637921004],[-113.64257812499999,46.37725420510028],[-109.9951171875,47.69497434186282],[-109.1162109375,46.28622391806708],[-103.9306640625,47.040182144806664],[-106.435546875,44.49650533109345],[-107.1826171875,45.42929873257377],[-105.9521484375,45.9511496866914],[-108.9404296875,45.706179285330855],[-105.8203125,41.80407814427237],[-108.10546875,41.343824581185686],[-110.0830078125,43.992814500489914],[-110.74218749999999,40.979898069620155],[-111.62109375,41.705728515237524],[-111.97265625,39.13006024213511],[-105.29296874999999,38.92522904714054],[-103.71093749999999,42.293564192170095],[-104.765625,43.389081939117496],[-103.22753906249999,45.644768217751924],[-100.986328125,42.261049162113856],[-103.798828125,36.914764288955936],[-112.412109375,36.06686213257888],[-115.00488281250001,38.06539235133249],[-113.37890625,41.44272637767212],[-119.091796875,37.54457732085582],[-123.96972656249999,38.61687046392973],[-121.728515625,41.07935114946899],[-126.21093749999999,43.13306116240612],[-127.3095703125,45.460130637921004],[-124.27734374999999,47.81315451752768],[-118.037109375,47.81315451752768],[-124.01367187499999,45.85941212790755],[-124.67285156250001,43.929549935614595],[-118.740234375,41.37680856570233],
    [-120.76171875,38.788345355085625]]]}}]}
    "#;

    let geojson: GeoJson = geojson_string.parse::<GeoJson>().unwrap();

    println!("geojson {:?}", geojson);

    // let mut tilescoverage: Vec<Tile> = vec![];
    let tilescoverage = geojson2tiles(geojson, 9).unwrap();
    println!("SHIT: {:?}", tilescoverage);

    // let tiles_set: HashSet<Tile> = tilescoverage.into_iter().collect();

    let tiles_set = tilescoverage;
    let expected_set: HashSet<Tile> = expected.into_iter().collect();

    println!("eq: {}", tiles_set == expected_set);
    // Find common elements (intersection)
    let common: HashSet<_> = tiles_set.intersection(&expected_set).cloned().collect();
    println!("Common elements (intersection): {:?}", common);

    // Find elements only in expected_set (expected_set \ tiles_set)
    let expected_only: HashSet<_> =
        expected_set.difference(&tiles_set).cloned().collect();
    println!("Elements only in expected_set: {:?}", expected_only);

    // Find elements only in tiles_set (tiles_set \ expected_set)
    let tiles_only: HashSet<_> = tiles_set.difference(&expected_set).cloned().collect();
    println!("Elements only in tiles_set: {:?}", tiles_only);
}
