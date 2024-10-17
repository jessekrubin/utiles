use crate::{UtilesError, UtilesResult};
use geojson::GeoJson;
use std::collections::HashSet;
use utiles_core::{from_id, lnglat2tile_frac, tile, to_id, utile, Tile};

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::similar_names)]
fn line_string_cover(
    tile_hash: &mut HashSet<u64>,
    coords: &[(f64, f64)],
    maxzoom: u8,
    mut ring: Option<&mut Vec<(u32, u32)>>,
) {
    let mut prev_x: Option<u32> = None;
    let mut prev_y: Option<u32> = None;

    let n = 1u32 << maxzoom; // Number of tiles at this zoom level

    for i in 0..coords.len() - 1 {
        let start_coord = coords[i];
        let stop_coord = coords[i + 1];

        let (x0f, y0f, _) = lnglat2tile_frac(start_coord.0, start_coord.1, maxzoom);
        let (x1f, y1f, _) = lnglat2tile_frac(stop_coord.0, stop_coord.1, maxzoom);

        let dx = x1f - x0f;
        let dy = y1f - y0f;

        if dx == 0.0 && dy == 0.0 {
            continue;
        }

        let sx = dx.signum();
        let sy = dy.signum();

        let mut x = x0f.floor() as i64;
        let mut y = y0f.floor() as i64;

        let tdx = if dx == 0.0 {
            f64::INFINITY
        } else {
            (sx / dx).abs()
        };
        let tdy = if dy == 0.0 {
            f64::INFINITY
        } else {
            (sy / dy).abs()
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

        // Wrap x and clamp y to valid ranges
        x = x.rem_euclid(i64::from(n));
        y = y.clamp(0, i64::from(n - 1));

        let x_u32 = x as u32;
        let y_u32 = y as u32;
        if prev_x != Some(x_u32) || prev_y != Some(y_u32) {
            tile_hash.insert(to_id(x_u32, y_u32, maxzoom));
            if let Some(ring) = &mut ring {
                if prev_y != Some(y_u32) {
                    ring.push((x_u32, y_u32));
                }
            }
            prev_x = Some(x_u32);
            prev_y = Some(y_u32);
        }
        // if prev_x.is_none()
        //     || prev_y.is_none()
        //     || x_u32 != prev_x.unwrap()
        //     || y_u32 != prev_y.unwrap()
        // {
        //     tile_hash.insert(to_id(x_u32, y_u32, maxzoom));
        //     if let Some(ring) = &mut ring {
        //         if prev_y != Some(y_u32) {
        //             ring.push((x_u32, y_u32));
        //         }
        //     }
        //     prev_x = Some(x_u32);
        //     prev_y = Some(y_u32);
        // }

        while t_max_x < 1.0 || t_max_y < 1.0 {
            if t_max_x < t_max_y {
                t_max_x += tdx;
                x += sx as i64;
            } else {
                t_max_y += tdy;
                y += sy as i64;
            }

            // Wrap x and clamp y
            x = x.rem_euclid(i64::from(n));
            y = y.clamp(0, i64::from(n - 1));

            let x_u32 = x as u32;
            let y_u32 = y as u32;

            tile_hash.insert(to_id(x_u32, y_u32, maxzoom));
            if let Some(ring) = &mut ring {
                if prev_y != Some(y_u32) {
                    ring.push((x_u32, y_u32));
                }
            }
            prev_x = Some(x_u32);
            prev_y = Some(y_u32);
        }
    }
}

fn polygon_cover(
    tile_hash: &mut HashSet<u64>,
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
            tile_array.push(utile!(x, y, zoom));
        }
        i += 2;
    }
}
fn append_hash_tiles(
    tile_hash: &HashSet<u64>,
    tiles: &mut Vec<Tile>,
) -> UtilesResult<()> {
    for id in tile_hash {
        let tile = from_id(*id)?;
        tiles.push(tile);
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
                tile_hash.insert(to_id(tile.x, tile.y, tile.z));
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
pub fn geojson2tiles(gj: &GeoJson, zoom: u8) -> UtilesResult<HashSet<Tile>> {
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
    Ok(tilescoverage)
}
