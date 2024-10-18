//! Tile cover for geojson object(s) based on mapbox's tile-cover alg
use crate::server::UtilesServerConfig;
use crate::{UtilesError, UtilesResult};
use geojson::GeoJson;
use std::collections::HashSet;
use tracing::debug;
use utiles_core::{lnglat2tile_frac, simplify, tile, utile, Tile};

#[allow(clippy::cast_precision_loss)]
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

#[allow(clippy::cast_precision_loss)]
fn polygon_cover(
    tile_hash: &mut HashSet<Tile>,
    tile_array: &mut Vec<Tile>,
    geom: &[Vec<(f64, f64)>],
    zoom: u8,
) {
    use std::collections::BTreeMap;

    let mut scanline_intersections: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    for element in geom {
        let mut ring = Vec::new();
        line_string_cover(tile_hash, element, zoom, Some(&mut ring));
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

            // Calculate the range of y values to process
            let ymin = y0.min(y1);
            let ymax = y0.max(y1);

            let dx = x1 - x0;
            let dy = y1 - y0;

            for y in ymin..ymax {
                // Calculate the intersection x coordinate
                let t = (y - y0) as f64 / dy as f64;
                let x = x0 as f64 + t * dx as f64;
                let x = x.floor() as u32;

                // Add the intersection to the scanline
                scanline_intersections
                    .entry(y as u32)
                    .or_insert_with(Vec::new)
                    .push(x);
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
                if !tile_hash.contains(&tile) {
                    tile_array.push(tile);
                }
            }

            i += 2;
        }
    }
}

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
            Ok::<(), UtilesError>(())
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
        geojson::Value::GeometryCollection(gjcoll) => {
            for geom in gjcoll {
                let recurse_res = geom2tiles(geom, zoom)?;
                tile_hash.extend(recurse_res);
            }
            Ok(())
        }
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
