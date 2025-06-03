//! Tile cover for geojson object(s) based on mapbox's tile-cover alg
use crate::{Result, UtilesCoverError};
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
    // Collect all x-intersections per scanline (y)
    let mut scanlines: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    for ring_pts in geom {
        // First collect the polygon boundary into `boundary` as tile-edge samples
        let mut boundary: Vec<(u32, u32)> = Vec::new();
        line_string_cover(tiles_set, ring_pts, zoom, Some(&mut boundary));
        if boundary.is_empty() {
            continue;
        }

        // Iterate each edge, including the closing edge from last→first
        let iter = boundary
            .windows(2)
            .map(|w| (w[0], w[1]))
            .chain(std::iter::once((boundary[boundary.len() - 1], boundary[0])));

        for ((x0f, y0f), (x1f, y1f)) in iter {
            let x0 = x0f as i32;
            let y0 = y0f as i32;
            let x1 = x1f as i32;
            let y1 = y1f as i32;

            // skip fully horizontal edges
            if y0 == y1 {
                continue;
            }

            // y ranges over the scanlines this edge crosses
            let (ymin, ymax) = (y0.min(y1), y0.max(y1));
            let dx = x1 - x0;
            let dy = y1 - y0;

            for y in ymin..ymax {
                // parametric t along the edge at integer y
                let t = (y - y0) as f64 / dy as f64;
                let x = (x0 as f64 + t * dx as f64).floor() as u32;
                scanlines.entry(y as u32).or_default().push(x);
            }
        }
    }

    // Fill in between pairs of intersections on each scanline
    for (y, mut xs) in scanlines {
        xs.sort_unstable();
        // take (x_start, x_end) from each pair of sorted intersections
        for pair in xs.chunks(2).filter(|c| c.len() == 2) {
            let x_start = pair[0];
            let x_end = pair[1];
            for x in x_start..x_end {
                tiles_set.insert(Tile::new(x, y, zoom));
            }
        }
    }
}

fn geom2tiles(geom: &geojson::Geometry, zoom: u8) -> Result<Vec<Tile>> {
    let mut tiles_set = HashSet::new();
    let res = match &geom.value {
        geojson::Value::Point(coords) => {
            let tile = tile(coords[0], coords[1], zoom, None)?;
            tiles_set.insert(tile);
            Ok::<(), UtilesCoverError>(())
        }
        geojson::Value::MultiPoint(coords_list) => {
            coords_list
                .iter()
                .map(|coords| {
                    tile(coords[0], coords[1], zoom, None)
                        .map_err(|e| UtilesCoverError::from(e))
                })
                .collect::<Result<HashSet<_>>>()
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
) -> Result<HashSet<Tile>> {
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
