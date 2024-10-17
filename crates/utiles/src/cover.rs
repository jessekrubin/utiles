use crate::{UtilesError, UtilesResult};
use geojson::GeoJson;
use std::collections::HashSet;
use tracing::debug;
use utiles_core::{lnglat2tile_frac, simplify, tile, utile, Tile};
const EPSILON: f64 = 1e-14; // Small value to account for floating-point precision

fn to_id(x: u32, y: u32, z: u8) -> u64 {
    let dim = 2u64 * (1u64 << z);
    ((dim * y as u64 + x as u64) * 32u64) + z as u64
}

fn from_id(id: u64) -> Tile {
    let z = (id % 32) as u8;
    let dim = 2u64 * (1u64 << z);
    let xy = (id - z as u64) / 32u64;
    let x = (xy % dim) as u32;
    let y = ((xy - x as u64) / dim) as u32;
    utile!(x, y, z)
}

// fn line_string_cover(tile_hash: &mut HashSet<Tile>, coords: &[(f64, f64)], zoom: u8) {
//     for i in 0..coords.len() - 1 {
//         let (x0f, y0f, _) = lnglat2tile_frac(coords[i].0, coords[i].1, zoom);
//         let (x1f, y1f, _) = lnglat2tile_frac(coords[i + 1].0, coords[i + 1].1, zoom);
//
//         let x0 = x0f.floor() as i32;
//         let y0 = y0f.floor() as i32;
//         let x1 = x1f.floor() as i32;
//         let y1 = y1f.floor() as i32;
//
//         bresenham_line(x0, y0, x1, y1, zoom, tile_hash);
//     }
// }
//
// fn bresenham_line(
//     x0: i32,
//     y0: i32,
//     x1: i32,
//     y1: i32,
//     zoom: u8,
//     tile_hash: &mut HashSet<Tile>,
// ) {
//     let mut x = x0;
//     let mut y = y0;
//     let dx = (x1 - x0).abs();
//     let dy = -(y1 - y0).abs();
//     let sx = if x0 < x1 { 1 } else { -1 };
//     let sy = if y0 < y1 { 1 } else { -1 };
//     let mut err = dx + dy;
//
//     loop {
//         if x >= 0 && y >= 0 {
//             let tile = utile!(x as u32, y as u32, zoom);
//             tile_hash.insert(tile);
//         }
//
//         if x == x1 && y == y1 {
//             break;
//         }
//
//         let e2 = 2 * err;
//         if e2 >= dy {
//             err += dy;
//             x += sx;
//         }
//         if e2 <= dx {
//             err += dx;
//             y += sy;
//         }
//     }
// }
//

// #[allow(clippy::cast_precision_loss)]
#[allow(clippy::similar_names)]
fn line_string_cover(
    tile_hash: &mut HashSet<Tile>,
    coords: &[(f64, f64)],
    maxzoom: u8,
    mut ring: Option<&mut Vec<(u32, u32)>>,
) {
    let mut prev_x: Option<i64> = None;
    let mut prev_y: Option<i64> = None;
    let mut y_value: Option<i64> = None;

    let n = 1u32 << maxzoom; // Number of tiles at this zoom level
    let minxy = (1u32 << maxzoom) - 1; // Maximum valid tile index at this zoom level

    for i in 0..coords.len() - 1 {
        let start_coord = coords[i];
        let stop_coord = coords[i + 1];

        let (x0f, y0f, _) = lnglat2tile_frac(start_coord.0, start_coord.1, maxzoom);
        let (x1f, y1f, _) = lnglat2tile_frac(stop_coord.0, stop_coord.1, maxzoom);

        let dx = x1f - x0f;
        let dy = y1f - y0f;

        // Directly check for zero movement
        if dx == 0.0 && dy == 0.0 {
            continue;
        }

        let sx = dx.signum() as i64;
        let sy = dy.signum() as i64;

        let mut x = x0f.floor() as i64;
        let mut y = y0f.floor() as i64;
        y_value = Some(y);

        let tdx = if dx == 0.0 {
            f64::INFINITY
        } else {
            (sx as f64 / dx).abs()
        };
        let tdy = if dy == 0.0 {
            f64::INFINITY
        } else {
            (sy as f64 / dy).abs()
        };

        let mut t_max_x = if dx == 0.0 {
            f64::INFINITY
        } else {
            ((if dx > 0.0 { 1.0 } else { 0.0 } + x as f64 - x0f) / dx).abs()
        };
        let mut t_max_y = if dy == 0.0 {
            f64::INFINITY
        } else {
            ((if dy > 0.0 { 1.0 } else { 0.0 } + y as f64 - y0f) / dy).abs()
        };

        // Remove the initial boundary check

        // Add initial tile
        if prev_x != Some(x) || prev_y != Some(y) {
            let tile = utile!(x as u32, y as u32, maxzoom);
            tile_hash.insert(tile);
            if let Some(ring) = &mut ring {
                if prev_y != Some(y) {
                    ring.push((x as u32, y as u32));
                }
            }
            prev_x = Some(x);
            prev_y = Some(y);
        }

        while t_max_x < 1.0 || t_max_y < 1.0 {
            if t_max_x < t_max_y {
                t_max_x += tdx;
                x += sx;
            } else {
                t_max_y += tdy;
                y += sy;
            }

            // Check if x or y is outside valid tile ranges
            if x < -1 || y < -5 || x > i64::from(minxy) + 1 || y > i64::from(minxy) + 1
            {
                break;
            }

            if prev_x != Some(x) || prev_y != Some(y) {
                let tile = utile!(x as u32, y as u32, maxzoom);
                tile_hash.insert(tile);
                if let Some(ring) = &mut ring {
                    if prev_y != Some(y) {
                        ring.push((x as u32, y as u32));
                    }
                }
                prev_x = Some(x);
                prev_y = Some(y);
            }
        }
    }

    // Adjust the ring if needed
    if let Some(ring) = &mut ring {
        if let (Some(first_ring), Some(y_value)) = (ring.first(), y_value) {
            if y_value == i64::from(first_ring.1) {
                ring.pop();
            }
        }
    }
}

fn polygon_cover(
    tile_hash: &mut HashSet<Tile>,
    tile_array: &mut Vec<Tile>,
    geom: &[Vec<(f64, f64)>],
    zoom: u8,
) {
    let mut intersections = Vec::new();
    for element in geom {
        let mut ring = Vec::new();
        line_string_cover(tile_hash, element, zoom, Some(&mut ring));
        let len = ring.len();
        for j in 0..len {
            let k = if j == 0 { len - 1 } else { j - 1 };
            let m = (j + 1) % len;

            let ring_j = ring[j];
            let ring_k = ring[k];
            let ring_m = ring[m];

            let y = ring_j.1;

            if (y > ring_k.1 || y > ring_m.1) // Not local minimum
                && (y < ring_k.1 || y < ring_m.1) // Not local maximum
                && y != ring_m.1
            {
                intersections.push(ring_j);
            }
        }
    }
    intersections.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    let mut i = 0;
    while i < intersections.len() {
        let y = intersections[i].1;
        let min_x = intersections[i].0 + 1;
        if i + 1 >= intersections.len() {
            break;
        }
        let max_x = intersections[i + 1].0;
        for x in min_x..max_x {
            let tile = utile!(x.try_into().unwrap(), y.try_into().unwrap(), zoom);
            tile_array.push(tile);
        }
        i += 2;
    }
}

// fn polygon_coverv1(
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
fn append_hash_tiles(
    tile_hash: &HashSet<Tile>,
    tiles: &mut Vec<Tile>,
) -> UtilesResult<()> {
    for id in tile_hash {
        // let tile = from_id(*id);
        tiles.push(*id);
    }
    Ok(())
}

fn geom2tiles(geom: &geojson::Geometry, zoom: u8) -> UtilesResult<Vec<Tile>> {
    let mut tile_hash = HashSet::new();
    let mut tiles = Vec::new();
    let res = match &geom.value {
        geojson::Value::Point(coords) => {
            let tile = tile(coords[0], coords[1], zoom, None)?;
            tiles.push(tile);
            Ok(())
        }
        geojson::Value::MultiPoint(coords_list) => {
            for coords in coords_list {
                let tile = tile(coords[0], coords[1], zoom, None)?;
                tile_hash.insert(tile);
                // tile_hash.insert(to_id(tile.x, tile.y, tile.z));
            }
            Ok(())
        }
        geojson::Value::LineString(coords_list) => {
            let coords: Vec<(f64, f64)> =
                coords_list.iter().map(|c| (c[0], c[1])).collect();
            line_string_cover(&mut tile_hash, &coords, zoom, None);
            Ok(())
        }
        geojson::Value::MultiLineString(coords_lists) => {
            for coords_list in coords_lists {
                let coords: Vec<(f64, f64)> =
                    coords_list.iter().map(|c| (c[0], c[1])).collect();
                line_string_cover(&mut tile_hash, &coords, zoom, None);
            }
            Ok(())
        }
        geojson::Value::Polygon(coords_lists) => {
            let coords: Vec<Vec<(f64, f64)>> = coords_lists
                .iter()
                .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
                .collect();
            polygon_cover(&mut tile_hash, &mut tiles, &coords, zoom);
            Ok(())
        }
        geojson::Value::MultiPolygon(coords_list_of_lists) => {
            for coords_lists in coords_list_of_lists {
                let coords: Vec<Vec<(f64, f64)>> = coords_lists
                    .iter()
                    .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
                    .collect();
                polygon_cover(&mut tile_hash, &mut tiles, &coords, zoom);
            }
            Ok(())
        }
        geojson::Value::GeometryCollection(_) => Err(UtilesError::Unsupported(
            "Unsupported geometry type".to_string(),
        )),
    };
    res?;
    append_hash_tiles(&tile_hash, &mut tiles)?;
    Ok(tiles)
}
pub fn geojson2tiles(
    gj: &GeoJson,
    zoom: u8,
    minzoom: Option<u8>,
) -> UtilesResult<HashSet<Tile>> {
    let mut tilescoverage: HashSet<Tile> = HashSet::new();

    match gj {
        GeoJson::FeatureCollection(ref ctn) => {
            for feature in &ctn.features {
                if let Some(ref geom) = feature.geometry {
                    let cov = geom2tiles(geom, zoom)?;
                    tilescoverage.extend(cov);
                }
            }
        }
        GeoJson::Feature(ref feature) => {
            if let Some(ref geom) = feature.geometry {
                let cov = geom2tiles(geom, zoom)?;
                tilescoverage.extend(cov);
            }
        }
        GeoJson::Geometry(ref geom) => {
            let cov = geom2tiles(geom, zoom)?;
            tilescoverage.extend(cov);
        }
    }

    match minzoom {
        Some(z) => {
            debug!("minzoom: {}", z);
            let cov = simplify(&tilescoverage, Some(z));
            Ok(cov)
        }
        None => Ok(tilescoverage),
    }
}
