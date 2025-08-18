//! Tile cover for geojson object(s) based on mapbox's tile-cover alg
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
use crate::Result;
use std::collections::{BTreeMap, HashSet};
use utiles_core::{Tile, lnglat2tile_frac, simplify, tile, utile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeoTypesCoverOptions {
    pub zoom: u8,
    pub minzoom: Option<u8>,
}

impl From<u8> for GeoTypesCoverOptions {
    fn from(zoom: u8) -> Self {
        Self {
            zoom,
            minzoom: None,
        }
    }
}

#[expect(clippy::cast_precision_loss)]
#[expect(clippy::similar_names)]
fn line_string_cover(
    tiles_set: &mut HashSet<Tile>,
    ls: &geo_types::LineString<f64>,
    maxzoom: u8,
    mut ring: Option<&mut Vec<(u32, u32)>>,
) {
    if ls.0.len() < 2 {
        return;
    }
    let mut prev_x: Option<i64> = None;
    let mut prev_y: Option<i64> = None;
    let mut y_value: Option<i64> = None;
    let minxy = (1u32 << maxzoom) - 1; // Maximum valid tile index at this zoom level

    // for segment in coords.windows(2) {
    for segment in ls.lines() {
        // let start_coord = segment
        let start_coord = segment.start;
        let stop_coord = segment.end;

        let (x0f, y0f, _) = lnglat2tile_frac(start_coord.x, start_coord.y, maxzoom);
        let (x1f, y1f, _) = lnglat2tile_frac(stop_coord.x, stop_coord.y, maxzoom);

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
            if let Some(ring) = &mut ring
                && prev_y != Some(y)
            {
                ring.push((x as u32, y as u32));
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
                if let Some(ring) = &mut ring
                    && prev_y != Some(y)
                {
                    ring.push((x as u32, y as u32));
                }
                prev_x = Some(x);
                prev_y = Some(y);
            }

            max_it -= 1; // Decrement the number of steps remaining
        }
    }

    // adjust the ring if needed
    if let Some(ring) = &mut ring
        && let (Some(first_ring), Some(y_value)) = (ring.first(), y_value)
        && y_value == i64::from(first_ring.1)
    {
        ring.pop();
    }
}

fn polygon_cover(
    tiles_set: &mut HashSet<Tile>,
    geom: &geo_types::Polygon<f64>,
    zoom: u8,
) {
    // Collect all x-intersections per scanline (y)
    let mut scanlines: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    let rings = vec![geom.exterior()] // Start with the exterior ring
        .into_iter()
        .chain(geom.interiors().iter()); // Add any interior rings

    for ring_pts in rings {
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
                let t = f64::from(y - y0) / f64::from(dy);
                let x = t.mul_add(f64::from(dx), f64::from(x0)).floor() as u32;
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
            tiles_set.extend((x_start..=x_end).map(|x| Tile::new(x, y, zoom)));
        }
    }
}

fn gt_geometry2tiles(
    geom: &geo_types::Geometry,
    opts: GeoTypesCoverOptions,
) -> Result<HashSet<Tile>> {
    let mut tilescoverage: HashSet<Tile> = HashSet::new();

    match geom {
        geo_types::Geometry::Point(pt) => {
            let tile = tile(pt.x(), pt.y(), opts.zoom, None)?;
            tilescoverage.insert(tile);
        }
        geo_types::Geometry::MultiPoint(pts) => {
            let it = pts
                .iter()
                .filter_map(|pt| tile(pt.x(), pt.y(), opts.zoom, None).ok());
            tilescoverage.extend(it);
        }
        geo_types::Geometry::Line(ln) => {
            let ls = geo_types::LineString::from(ln);
            line_string_cover(&mut tilescoverage, &ls, opts.zoom, None);
        }
        geo_types::Geometry::LineString(ls) => {
            line_string_cover(&mut tilescoverage, ls, opts.zoom, None);
        }
        geo_types::Geometry::MultiLineString(mls) => {
            mls.iter().for_each(|ls| {
                line_string_cover(&mut tilescoverage, ls, opts.zoom, None);
            });
        }
        geo_types::Geometry::Polygon(poly) => {
            polygon_cover(&mut tilescoverage, poly, opts.zoom);
        }
        geo_types::Geometry::MultiPolygon(mpoly) => {
            for poly in mpoly.iter() {
                polygon_cover(&mut tilescoverage, poly, opts.zoom);
            }
        }
        geo_types::Geometry::GeometryCollection(gjcoll) => {
            for g in gjcoll {
                tilescoverage.extend(gt_geometry2tiles(g, opts)?);
            }
        }
        geo_types::Geometry::Rect(r) => {
            let poly = geo_types::Polygon::from(*r);
            polygon_cover(&mut tilescoverage, &poly, opts.zoom);
        }
        geo_types::Geometry::Triangle(t) => {
            let poly = geo_types::Polygon::from(*t);
            polygon_cover(&mut tilescoverage, &poly, opts.zoom);
        }
    }

    match opts.minzoom {
        Some(z) => {
            let cov = simplify(&tilescoverage, Some(z));
            Ok(cov)
        }
        None => Ok(tilescoverage),
    }
}

/// Convert a `geo_types::Geometry` to a set of tiles at the specified zoom level.
///
/// # Errors
///
/// If the `GeoJSON` object is invalid or if the conversion fails due to
/// projecting coordinate issues.
pub fn geometry2tiles<T>(geom: &geo_types::Geometry, opts: T) -> Result<HashSet<Tile>>
where
    T: Into<GeoTypesCoverOptions>,
{
    let opts = opts.into();
    gt_geometry2tiles(geom, opts)
}
