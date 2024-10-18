//! Tile cover for geojson object(s) based on mapbox's tile-cover alg
use crate::{UtilesError, UtilesResult};
use geojson::GeoJson;
use std::collections::{BTreeMap, HashSet};
use tracing::debug;
use utiles_core::{lnglat2tile_frac, simplify, tile, utile, Tile};

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::similar_names)]
fn line_string_cover(
    tiles_set: &mut HashSet<Tile>,
    coords: &[(f64, f64)],
    maxzoom: u8,
    mut ring: Option<&mut Vec<(u32, u32)>>,
) {
    let mut prev_x: Option<i64> = None;
    let mut prev_y: Option<i64> = None;
    let mut y_value: Option<i64> = None;
    let minxy = (1u32 << maxzoom) - 1; // Maximum valid tile index at this zoom level
    for segment in coords.windows(2) {
        let start_coord = segment[0];
        let stop_coord = segment[1];

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

        // Add initial tile
        if prev_x != Some(x) || prev_y != Some(y) {
            let tile = utile!(x as u32, y as u32, maxzoom);
            tiles_set.insert(tile);
            if let Some(ring) = &mut ring {
                if prev_y != Some(y) {
                    ring.push((x as u32, y as u32));
                }
            }
            prev_x = Some(x);
            prev_y = Some(y);
        }

        // the MAX number of tiles to check to dx.abs() + dy.abs()
        let mut max_it = (dx.abs() + dy.abs()) as i64;
        while (t_max_x < 1.0 || t_max_y < 1.0) && max_it >= 0 {
            if t_max_x < t_max_y {
                t_max_x += tdx;
                x += sx;
            } else {
                t_max_y += tdy;
                y += sy;
            }

            // Sanity check if x or y is outside valid tile ranges
            if x > i64::from(minxy) + 1 || y > i64::from(minxy) + 1 {
                break;
            }

            if prev_x != Some(x) || prev_y != Some(y) {
                let tile = utile!(x as u32, y as u32, maxzoom);
                tiles_set.insert(tile);
                if let Some(ring) = &mut ring {
                    if prev_y != Some(y) {
                        ring.push((x as u32, y as u32));
                    }
                }
                prev_x = Some(x);
                prev_y = Some(y);
            }

            max_it -= 1; // Decrement the number of steps remaining
        }
    }

    // adjust the ring if needed
    if let Some(ring) = &mut ring {
        if let (Some(first_ring), Some(y_value)) = (ring.first(), y_value) {
            if y_value == i64::from(first_ring.1) {
                ring.pop();
            }
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn polygon_cover(tiles_set: &mut HashSet<Tile>, geom: &[Vec<(f64, f64)>], zoom: u8) {
    let mut scanline_intersections: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    for element in geom {
        let mut ring = Vec::new();
        line_string_cover(tiles_set, element, zoom, Some(&mut ring));
        let len = ring.len();

        if len == 0 {
            continue;
        }

        let mut j = 0usize;
        let mut k = len - 1;

        while j < len {
            let ring_j = ring[j];
            let ring_k = ring[k];

            let x0 = ring_k.0 as i32;
            let y0 = ring_k.1 as i32;
            let x1 = ring_j.0 as i32;
            let y1 = ring_j.1 as i32;

            // Skip horizontal edges
            if y0 == y1 {
                k = j;
                j += 1;
                continue;
            }

            // range of y values
            let ymin = y0.min(y1);
            let ymax = y0.max(y1);

            let dx = x1 - x0;
            let dy = y1 - y0;

            for y in ymin..ymax {
                // add intersection x coordinate
                let t = f64::from(y - y0) / f64::from(dy);
                let x = f64::from(x0) + t * f64::from(dx);
                let x = x.floor() as u32;

                // Add intersection point
                scanline_intersections.entry(y as u32).or_default().push(x);
            }
            k = j;
            j += 1;
        }
    }

    // Now process each scanline
    for (&y, xs) in &scanline_intersections {
        let mut xs = xs.clone();
        xs.sort_unstable();

        let mut i = 0;
        while i + 1 < xs.len() {
            let x_start = xs[i];
            let x_end = xs[i + 1];

            for x in x_start..x_end {
                let tile = Tile::new(x, y, zoom);
                tiles_set.insert(tile);
                // if !tiles_set.contains(&tile) {
                //     tiles_vec.push(tile);
                // }
            }

            i += 2;
        }
    }
}

fn geom2tiles(geom: &geojson::Geometry, zoom: u8) -> UtilesResult<Vec<Tile>> {
    let mut tiles_set = HashSet::new();
    let res = match &geom.value {
        geojson::Value::Point(coords) => {
            let tile = tile(coords[0], coords[1], zoom, None)?;
            tiles_set.insert(tile);
            Ok::<(), UtilesError>(())
        }
        geojson::Value::MultiPoint(coords_list) => {
            coords_list
                .iter()
                .map(|coords| tile(coords[0], coords[1], zoom, None))
                .collect::<Result<HashSet<_>, _>>()
                .map(|set| {
                    tiles_set.extend(set);
                })?;
            Ok(())
        }
        geojson::Value::LineString(coords_list) => {
            let coords: Vec<(f64, f64)> =
                coords_list.iter().map(|c| (c[0], c[1])).collect();
            line_string_cover(&mut tiles_set, &coords, zoom, None);
            Ok(())
        }
        geojson::Value::MultiLineString(coords_lists) => {
            for coords_list in coords_lists {
                let coords: Vec<(f64, f64)> =
                    coords_list.iter().map(|c| (c[0], c[1])).collect();
                line_string_cover(&mut tiles_set, &coords, zoom, None);
            }
            Ok(())
        }
        geojson::Value::Polygon(coords_lists) => {
            let coords: Vec<Vec<(f64, f64)>> = coords_lists
                .iter()
                .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
                .collect();
            polygon_cover(&mut tiles_set, &coords, zoom);
            Ok(())
        }
        geojson::Value::MultiPolygon(coords_list_of_lists) => {
            for coords_lists in coords_list_of_lists {
                let coords: Vec<Vec<(f64, f64)>> = coords_lists
                    .iter()
                    .map(|ring| ring.iter().map(|c| (c[0], c[1])).collect())
                    .collect();
                polygon_cover(&mut tiles_set, &coords, zoom);
            }
            Ok(())
        }
        geojson::Value::GeometryCollection(gjcoll) => {
            for geom in gjcoll {
                let recurse_res = geom2tiles(geom, zoom)?;
                tiles_set.extend(recurse_res);
            }
            Ok(())
        }
    };
    res?;
    let tiles_vec = tiles_set.into_iter().collect();
    Ok(tiles_vec)
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
