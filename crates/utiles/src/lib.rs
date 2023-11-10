#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::style)]

use std::collections::{HashMap, HashSet};
use std::num::FpCategory;
use std::{error::Error, f64::consts::PI};

use bbox::{BBox, WebMercatorBbox};
use constants::{EARTH_CIRCUMFERENCE, EARTH_RADIUS, LL_EPSILON};
use geo_types::coord;

pub use lnglat::LngLat;
use sibling_relationship::SiblingRelationship;
pub use tile::Tile;
use tile_range::{TileRange, TileRanges};
use zoom::ZoomOrZooms;

pub mod bbox;
pub mod constants;
pub mod geojson;
pub mod libtiletype;
pub mod lnglat;
pub mod mbtiles;
pub mod parsing;
pub mod pmtiles;
pub mod projection;
pub mod sibling_relationship;
pub mod tile;
mod tile_feature;
pub mod tile_range;
mod tile_tuple;
pub mod tilejson;
pub mod traits;
pub mod zoom;

/// Tile macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to to figure out how to write a macro? yes
#[macro_export]
macro_rules! utile {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, $y, $z)
    };
}

#[must_use]
pub fn ul(x: u32, y: u32, z: u8) -> LngLat {
    let (lon_deg, lat_deg) = ult(x, y, z);
    LngLat {
        xy: coord! {x: lon_deg, y: lat_deg},
    }
}

#[must_use]
pub fn ll(x: u32, y: u32, z: u8) -> LngLat {
    ul(x, y + 1, z)
}

#[must_use]
pub fn ur(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y, z)
}

#[must_use]
pub fn lr(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y + 1, z)
}
#[must_use]
pub fn ult(x: u32, y: u32, z: u8) -> (f64, f64) {
    let z2 = f64::from(2_u32.pow(u32::from(z)));
    let lon_deg = (f64::from(x) / z2) * 360.0 - 180.0;
    let lat_rad = ((1.0 - 2.0 * f64::from(y) / z2) * PI).sinh().atan();

    let lat_deg = lat_rad.to_degrees();
    (lon_deg, lat_deg)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XY {
    pub x: u32,
    pub y: u32,
}

/// Truncate a bounding box to the valid range of longitude and latitude.
#[must_use]
pub fn bbox_truncate(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    truncate: Option<bool>,
) -> (f64, f64, f64, f64) {
    let trunc = truncate.unwrap_or(false);
    let mut west = west;
    let mut east = east;
    let mut south = south;
    let mut north = north;
    if trunc {
        if west < -180.0 {
            west = -180.0;
        }
        if east > 180.0 {
            east = 180.0;
        }
        if south < -90.0 {
            south = -90.0;
        }
        if north > 90.0 {
            north = 90.0;
        }
    }
    (west, south, east, north)
}

#[must_use]
pub fn minmax(zoom: u32) -> (u32, u32) {
    (0, 2_u32.pow(zoom) - 1)
}

#[must_use]
pub fn valid(x: u32, y: u32, z: u8) -> bool {
    let (minx, maxx) = minmax(u32::from(z));
    let (miny, maxy) = minmax(u32::from(z));
    x >= minx && x <= maxx && y >= miny && y <= maxy
}

#[must_use]
pub fn flipy(y: u32, z: u8) -> u32 {
    2_u32.pow(u32::from(z)) - 1 - y
}

#[must_use]
pub fn bbox2zoom(bbox: (u32, u32, u32, u32)) -> u8 {
    let max_zoom = 28;
    let (west, south, east, north) = bbox;
    for z in 0..max_zoom {
        let mask = 1 << (32 - (z + 1));
        if (west & mask) != (east & mask) || (south & mask) != (north & mask) {
            return z;
        }
    }
    max_zoom
}

#[must_use]
pub fn bounds(x: u32, y: u32, z: u8) -> (f64, f64, f64, f64) {
    let ul_corner = ul(x, y, z);
    let lr_corner = ul(x + 1, y + 1, z);
    (
        ul_corner.lng(),
        lr_corner.lat(),
        lr_corner.lng(),
        ul_corner.lat(),
    )
}

#[must_use]
pub fn truncate_lng(lng: f64) -> f64 {
    if lng > 180.0 {
        180.0
    } else if lng < -180.0 {
        -180.0
    } else {
        lng
    }
}

#[must_use]
pub fn truncate_lat(lat: f64) -> f64 {
    if lat > 90.0 {
        90.0
    } else if lat < -90.0 {
        -90.0
    } else {
        lat
    }
}

#[must_use]
pub fn truncate_lnglat(lnglat: &LngLat) -> LngLat {
    LngLat {
        xy: coord! {x: truncate_lng(lnglat.lng()), y: truncate_lat(lnglat.lat())},
    }
}

pub fn _xy(
    lng: f64,
    lat: f64,
    truncate: Option<bool>,
) -> Result<(f64, f64), &'static str> {
    let (lng, lat) = if truncate.unwrap_or(false) {
        (truncate_lng(lng), truncate_lat(lat))
    } else {
        (lng, lat)
    };

    let x = lng / 360.0 + 0.5;
    let sinlat = (lat.to_radians()).sin();

    let temp = (1.0 + sinlat) / (1.0 - sinlat);
    match temp.classify() {
        FpCategory::Infinite | FpCategory::Nan => {
            Err("Y can not be computed: lat={lat}")
        }
        _ => {
            let y = 0.5 - 0.25 * (temp.ln()) / PI;
            Ok((x, y))
        }
    }
}

// pub fn _xy_og(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
//     let trunc = truncate.unwrap_or(false);
//
//     let mut lng = lng;
//     let mut lat = lat;
//     if trunc {
//         lng = truncate_lng(lng);
//         lat = truncate_lat(lat);
//     }
//     let x = (lng / 360.0) + 0.5;
//
//     let sinlat = lat.to_radians().sin();
//
//     let y_inner = (1.0 + sinlat) / (1.0 - sinlat);
//     let y = match (1.0 + sinlat) / (1.0 - sinlat) {
//         y if y.is_infinite() => {
//             panic!("Invalid latitude (inf): {lat:?}");
//         }
//         y if y.is_nan() => {
//             panic!("Invalid latitude (nan): {lat:?}");
//         }
//         // y => 0.5 - 0.25 * y.ln() / std::f64::consts::PI,
//         // y = 0.5 - 0.25 * math.log((1.0 + sinlat) / (1.0 - sinlat)) / math.pi
//         _y => 0.5 - 0.25 * y_inner.ln() / PI,
//     };
//     (x, y)
//     // y = 0.5 - 0.25 * math.log((1.0 + sinlat) / (1.0 - sinlat)) / math.pi
// }

#[must_use]
pub fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    let trunc = truncate.unwrap_or(false);
    let mut lng = lng;
    let mut lat = lat;
    if trunc {
        lng = truncate_lng(lng);
        lat = truncate_lat(lat);
    }
    let x = EARTH_RADIUS * lng.to_radians();
    let y = if lat == 90.0 {
        f64::INFINITY
    } else if lat == -90.0 {
        f64::NEG_INFINITY
    } else {
        // (1.0 + (lat.to_radians()).sin()) / (1.0 - (lat.to_radians()).sin())
        EARTH_RADIUS * (PI * 0.25 + 0.5 * lat.to_radians()).tan().ln()
    };
    (x, y)
}

#[must_use]
pub fn lnglat(x: f64, y: f64, truncate: Option<bool>) -> LngLat {
    let lng = x / EARTH_RADIUS * 180.0 / PI;
    let lat = (2.0 * (y / EARTH_RADIUS).exp().atan() - PI * 0.5) * 180.0 / PI;
    if truncate.is_some() {
        truncate_lnglat(&LngLat::new(lng, lat))
        // LngLat {
        //     lng: truncate_lng(lng),
        //     lat: truncate_lat(lat),
        // }
    } else {
        LngLat::new(lng, lat)
    }
}

#[must_use]
pub fn parent(x: u32, y: u32, z: u8, n: Option<u8>) -> Tile {
    let n = n.unwrap_or(0);
    if n == 0 {
        Tile {
            x: x >> 1,
            y: y >> 1,
            z: z - 1,
        }
    } else {
        parent(x >> 1, y >> 1, z - 1, Some(n - 1))
    }
}

#[must_use]
pub fn children(x: u32, y: u32, z: u8, zoom: Option<u8>) -> Vec<Tile> {
    let zoom = zoom.unwrap_or(z + 1);
    let tile = Tile { x, y, z };
    let mut tiles = vec![tile];
    while tiles[0].z < zoom {
        let (xtile, ytile, ztile) = (tiles[0].x, tiles[0].y, tiles[0].z);
        tiles.append(&mut vec![
            Tile::new(xtile * 2, ytile * 2, ztile + 1),
            Tile::new(xtile * 2 + 1, ytile * 2, ztile + 1),
            Tile::new(xtile * 2 + 1, ytile * 2 + 1, ztile + 1),
            Tile::new(xtile * 2, ytile * 2 + 1, ztile + 1),
        ]);
        tiles.remove(0);
    }
    tiles
}

#[must_use]
pub fn siblings(x: u32, y: u32, z: u8) -> Vec<Tile> {
    let sibrel = SiblingRelationship::from((x, y));
    match sibrel {
        SiblingRelationship::UpperLeft => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x + 1, y + 1, z),
        ],
        SiblingRelationship::UpperRight => vec![
            Tile::new(x - 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x - 1, y + 1, z),
        ],
        SiblingRelationship::LowerLeft => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y - 1, z),
            Tile::new(x + 1, y - 1, z),
        ],
        SiblingRelationship::LowerRight => vec![
            Tile::new(x - 1, y, z),
            Tile::new(x, y - 1, z),
            Tile::new(x - 1, y - 1, z),
        ],
    }
}

enum TileEdgeInfo {
    Bottom,
    BottomLeft,
    BottomRight,
    Left,
    Right,
    Top,
    TopLeft,
    TopRight,
    Middle,
}

fn _tile_edge_info(x: u32, y: u32, z: u8) -> TileEdgeInfo {
    if x == 0 && y == 0 {
        return TileEdgeInfo::TopLeft;
    }
    let max_xy = 2u32.pow(u32::from(z));
    if x == max_xy && y == 0 {
        return TileEdgeInfo::TopRight;
    }
    if x == 0 && y == max_xy {
        return TileEdgeInfo::BottomLeft;
    }

    if x == max_xy && y == max_xy {
        return TileEdgeInfo::BottomRight;
    }
    if x == 0 {
        return TileEdgeInfo::Left;
    }
    if x == max_xy {
        return TileEdgeInfo::Right;
    }
    if y == 0 {
        return TileEdgeInfo::Top;
    }
    if y == max_xy {
        return TileEdgeInfo::Bottom;
    }
    TileEdgeInfo::Middle
}

fn _neighbors_middle_tile(x: u32, y: u32, z: u8) -> Vec<Tile> {
    vec![
        Tile::new(x + 1, y, z),
        Tile::new(x, y + 1, z),
        Tile::new(x + 1, y + 1, z),
        Tile::new(x - 1, y, z),
        Tile::new(x, y - 1, z),
        Tile::new(x - 1, y - 1, z),
        Tile::new(x + 1, y - 1, z),
        Tile::new(x - 1, y + 1, z),
    ]
}

#[must_use]
pub fn neighbors(x: u32, y: u32, z: u8) -> Vec<Tile> {
    if z == 0 {
        return Vec::new();
    }
    let edge_info = _tile_edge_info(x, y, z);
    match edge_info {
        TileEdgeInfo::Middle => _neighbors_middle_tile(x, y, z),
        TileEdgeInfo::TopLeft => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x + 1, y + 1, z),
        ],
        TileEdgeInfo::TopRight => vec![
            Tile::new(x - 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x - 1, y + 1, z),
        ],
        TileEdgeInfo::BottomLeft => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y - 1, z),
            Tile::new(x + 1, y - 1, z),
        ],
        TileEdgeInfo::BottomRight => vec![
            Tile::new(x - 1, y, z),
            Tile::new(x, y - 1, z),
            Tile::new(x - 1, y - 1, z),
        ],
        TileEdgeInfo::Left => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x + 1, y + 1, z),
            Tile::new(x, y - 1, z),
            Tile::new(x + 1, y - 1, z),
        ],
        TileEdgeInfo::Right => vec![
            Tile::new(x - 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x - 1, y + 1, z),
            Tile::new(x, y - 1, z),
            Tile::new(x - 1, y - 1, z),
        ],
        TileEdgeInfo::Top => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y + 1, z),
            Tile::new(x + 1, y + 1, z),
            Tile::new(x - 1, y, z),
            Tile::new(x - 1, y + 1, z),
        ],
        TileEdgeInfo::Bottom => vec![
            Tile::new(x + 1, y, z),
            Tile::new(x, y - 1, z),
            Tile::new(x + 1, y - 1, z),
            Tile::new(x - 1, y, z),
            Tile::new(x - 1, y - 1, z),
        ],
    }
}

// tile = ut.Tile(486, 332, 10)
// expected = "0313102310"
/// Return the quadkey for a tile as a string.
/// # Examples
/// ```
/// use utiles::xyz2quadkey;
/// let quadkey = xyz2quadkey(486, 332, 10);
/// assert_eq!(quadkey, "0313102310");
/// ```
#[must_use]
pub fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    let mut quadkey = String::new();
    for i in (0..z).rev() {
        let mut digit = 0;
        let mask = 1 << i;
        if (x & mask) != 0 {
            digit += 1;
        }
        if (y & mask) != 0 {
            digit += 2;
        }
        quadkey.push_str(&digit.to_string());
    }
    quadkey
}

pub fn quadkey2xyz(quadkey: &str) -> Result<(u32, u32, u8), Box<dyn Error>> {
    let mut x = 0;
    let mut y = 0;
    let mut z = 0;
    for c in quadkey.chars() {
        x <<= 1;
        y <<= 1;
        z += 1;
        match c {
            '0' => {}
            '1' => {
                x += 1;
            }
            '2' => {
                y += 1;
            }
            '3' => {
                x += 1;
                y += 1;
            }
            _ => {
                Err("Invalid quadkey char found")?;
                // panic!("Invalid quadkey char: {}", c);
            }
        }
    }
    Ok((x, y, z))
}

pub fn quadkey2tile(quadkey: &str) -> Result<Tile, Box<dyn Error>> {
    let (x, y, z) = quadkey2xyz(quadkey)?;
    Ok(Tile::new(x, y, z))
}

impl From<Tile> for (u32, u32, u8) {
    fn from(tile: Tile) -> Self {
        (tile.x, tile.y, tile.z)
    }
}

#[must_use]
pub fn xyz2bbox(x: u32, y: u32, z: u8) -> WebMercatorBbox {
    let tile_size = EARTH_CIRCUMFERENCE / 2.0_f64.powi(i32::from(z));
    let left = f64::from(x) * tile_size - EARTH_CIRCUMFERENCE / 2.0;
    let right = left + tile_size;
    let top = EARTH_CIRCUMFERENCE / 2.0 - f64::from(y) * tile_size;
    let bottom = top - tile_size;
    WebMercatorBbox {
        left,
        bottom,
        right,
        top,
    }
}

#[must_use]
pub fn as_zooms(zoom_or_zooms: ZoomOrZooms) -> Vec<u8> {
    match zoom_or_zooms {
        ZoomOrZooms::Zoom(zoom) => {
            vec![zoom]
        }
        ZoomOrZooms::Zooms(zooms) => zooms,
    }
}

fn tiles_range_zoom(
    minx: u32,
    maxx: u32,
    miny: u32,
    maxy: u32,
    zoom: u8,
) -> impl Iterator<Item = (u32, u32, u8)> {
    (minx..=maxx).flat_map(move |i| (miny..=maxy).map(move |j| (i, j, zoom)))
}

#[must_use]
pub fn tile(lng: f64, lat: f64, zoom: u8, truncate: Option<bool>) -> Tile {
    Tile::from_lnglat_zoom(lng, lat, zoom, truncate)
}

#[must_use]
pub fn bounding_tile(bbox: BBox, truncate: Option<bool>) -> Tile {
    let (west, south, east, north) =
        bbox_truncate(bbox.west, bbox.south, bbox.east, bbox.north, truncate);
    let tmin = tile(west, north, 32, truncate);
    let tmax = tile(east - LL_EPSILON, south + LL_EPSILON, 32, truncate);

    let cell = (tmin.x, tmin.y, tmax.x, tmax.y);
    let z = bbox2zoom(cell);
    if z == 0 {
        return Tile::new(0, 0, 0);
    }

    let x = cell.0 >> (32 - z);
    let y = cell.1 >> (32 - z);
    Tile::new(x, y, z)
}
// fn tile_tupless_range(
//     minx: i32,
//     maxx: i32,
//     miny: i32,
//     maxy: i32,
//     zoom_or_zooms: ZoomOrZooms,
// ) -> impl Iterator<Item = (i32, i32, i32)> {
//     let zooms = as_zooms(zoom_or_zooms);
//     zooms
//         .into_iter()
//         .flat_map(move |zoom| tiles_range_zoom(minx, maxx, miny, maxy, zoom))
// }

// fn bounds2xy(bounds: (f64, f64, f64, f64), zoom: i32) -> (i32, i32, i32, i32) {
//     let (minx, miny) = xy(bounds.0, bounds.1, None);
//     let (maxx, maxy) = xy(bounds.2, bounds.3, None);
//     let z2 = 2.0_f64.powi(zoom);
//     let minx = (minx * z2).floor() as i32;
//     let miny = (miny * z2).floor() as i32;
//     let maxx = (maxx * z2).floor() as i32;
//     let maxy = (maxy * z2).floor() as i32;
//     (minx, miny, maxx, maxy)
// }

#[must_use]
pub fn tile_ranges(bounds: (f64, f64, f64, f64), zooms: ZoomOrZooms) -> TileRanges {
    let zooms = as_zooms(zooms);
    let bboxthing = BBox {
        north: bounds.3,
        south: bounds.1,
        east: bounds.2,
        west: bounds.0,
    };
    let bboxes: Vec<BBox> = bboxthing
        .bboxes()
        .into_iter()
        .map(|bbox| {
            // clip to web mercator extent
            BBox {
                north: bbox.north.min(85.051_129),
                south: bbox.south.max(-85.051_129),
                east: bbox.east.min(180.0),
                west: bbox.west.max(-180.0),
            }
        })
        .collect();
    let ranges: Vec<TileRange> = bboxes
        .into_iter()
        .flat_map(move |bbox| {
            let zooms = zooms.clone();
            zooms.into_iter().map(move |zoom| {
                let upper_left_lnglat = LngLat {
                    xy: coord! { x: bbox.west, y: bbox.north },
                };
                let lower_right_lnglat = LngLat {
                    xy: coord! { x: bbox.east, y: bbox.south },
                };
                let top_left_tile = Tile::from_lnglat_zoom(
                    upper_left_lnglat.lng(),
                    upper_left_lnglat.lat(),
                    zoom,
                    Some(false),
                );
                let bottom_right_tile = Tile::from_lnglat_zoom(
                    lower_right_lnglat.lng() - LL_EPSILON,
                    lower_right_lnglat.lat() + LL_EPSILON,
                    zoom,
                    Some(false),
                );
                TileRange::new(
                    top_left_tile.x,
                    bottom_right_tile.x,
                    top_left_tile.y,
                    bottom_right_tile.y,
                    zoom,
                )
            })
        })
        .collect();

    TileRanges::from(ranges)
}

#[must_use]
pub fn tiles_count(bounds: (f64, f64, f64, f64), zooms: ZoomOrZooms) -> u64 {
    let ranges = tile_ranges(bounds, zooms);
    ranges.length()
}

pub fn tiles(
    bounds: (f64, f64, f64, f64),
    zooms: ZoomOrZooms,
) -> impl Iterator<Item = Tile> {
    let zooms = as_zooms(zooms);
    let bboxthing = BBox {
        north: bounds.3,
        south: bounds.1,
        east: bounds.2,
        west: bounds.0,
    };
    let bboxes: Vec<BBox> = bboxthing
        .bboxes()
        .into_iter()
        .map(|bbox| {
            // clip to web mercator extent
            BBox {
                north: bbox.north.min(85.051_129),
                south: bbox.south.max(-85.051_129),
                east: bbox.east.min(180.0),
                west: bbox.west.max(-180.0),
            }
        })
        .collect();
    bboxes.into_iter().flat_map(move |bbox| {
        let zooms = zooms.clone();
        zooms.into_iter().flat_map(move |zoom| {
            let upper_left_lnglat = LngLat {
                xy: coord! { x: bbox.west, y: bbox.north },
            };
            let lower_right_lnglat = LngLat {
                xy: coord! { x: bbox.east, y: bbox.south },
            };
            let top_left_tile = Tile::from_lnglat_zoom(
                upper_left_lnglat.lng(),
                upper_left_lnglat.lat(),
                zoom,
                Some(false),
            );
            let bottom_right_tile = Tile::from_lnglat_zoom(
                lower_right_lnglat.lng() - LL_EPSILON,
                lower_right_lnglat.lat() + LL_EPSILON,
                zoom,
                Some(false),
            );
            tiles_range_zoom(
                top_left_tile.x,
                bottom_right_tile.x,
                top_left_tile.y,
                bottom_right_tile.y,
                zoom,
            )
            .map(move |(x, y, z)| Tile { x, y, z })
        })
    })
}

#[allow(dead_code)]
fn merge(merge_set: &HashSet<Tile>) -> (HashSet<Tile>, bool) {
    let mut upwards_merge: HashMap<Tile, HashSet<Tile>> = HashMap::new();
    for tile in merge_set {
        let tile_parent = tile.parent(None);
        let children_set = upwards_merge
            .entry(tile_parent)
            .or_insert_with(HashSet::new);
        children_set.insert(*tile);
    }
    let mut current_tileset: Vec<Tile> = Vec::new();
    let mut changed = false;
    for (supertile, children) in upwards_merge {
        if children.len() == 4 {
            current_tileset.push(supertile);
            changed = true;
        } else {
            current_tileset.extend(children);
        }
    }
    (current_tileset.into_iter().collect(), changed)
}

#[allow(dead_code)]
#[must_use]
pub fn simplify(tiles: HashSet<Tile>) -> HashSet<Tile> {
    // Parse tiles from the input sequence
    let mut _tiles = tiles.into_iter().collect::<Vec<Tile>>();

    _tiles.sort_by_key(|t| t.z);
    // Check to see if a tile and its parent both already exist.
    // Ensure that tiles are sorted by zoom so parents are encountered first.
    // If so, discard the child (it's covered in the parent)
    let mut root_set: HashSet<Tile> = HashSet::new();
    for tile in &_tiles {
        let mut is_new_tile = true;
        for i in 0..tile.z {
            let supertile = tile.parent(Some(i));
            if root_set.contains(&supertile) {
                is_new_tile = false;
                break;
            }
        }
        if is_new_tile {
            root_set.insert(*tile);
        }
    }

    // Repeatedly run merge until no further simplification is possible.
    let mut is_merging = true;
    while is_merging {
        let (new_set, changed) = merge(&root_set);
        root_set = new_set;
        is_merging = changed;
    }
    root_set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_or_zooms() {
        let z = as_zooms(1.into());
        assert_eq!(z, vec![1]);
        let z = as_zooms(vec![1, 2, 3].into());
        assert_eq!(z, vec![1, 2, 3]);
    }

    #[test]
    fn tiles_generator() {
        let bounds = (-105.0, 39.99, -104.99, 40.0);
        let tiles = tiles(bounds, vec![14].into());
        let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
        assert_eq!(tiles.collect::<Vec<Tile>>(), expect);
    }

    #[test]
    fn tiles_single_zoom() {
        let bounds = (-105.0, 39.99, -104.99, 40.0);
        let tiles = tiles(bounds, 14.into());
        let expect = vec![Tile::new(3413, 6202, 14), Tile::new(3413, 6203, 14)];
        assert_eq!(tiles.collect::<Vec<Tile>>(), expect);

        let ntiles = tiles_count(bounds, 14.into());
        assert_eq!(ntiles, 2);
    }

    #[test]
    fn tiles_anti_meridian() {
        let bounds = (175.0, 5.0, -175.0, 10.0);
        let mut tiles: Vec<Tile> = tiles(bounds, 2.into()).collect();
        tiles.sort();
        let mut expected = vec![Tile::new(3, 1, 2), Tile::new(0, 1, 2)];
        expected.sort();
        assert_eq!(tiles, expected);
    }

    #[test]
    fn tile_is_valid() {
        let tile = Tile::new(0, 0, 0);
        assert!(tile.valid());
        let tile = Tile::new(1, 0, 0);
        assert!(!tile.valid());
        let tile = Tile::new(0, 1, 0);
        assert!(!tile.valid());
        let tile = Tile::new(0, 0, 1);
        assert!(tile.valid());
        let tile = Tile::new(1, 1, 1);
        assert!(tile.valid());

        // invalid tile
        let tile = Tile::new(1, 1, 0);
        assert!(!tile.valid());
        let _tile = Tile::new(1, 234, 1);
        assert!(!_tile.valid());
    }

    #[test]
    fn test_macro() {
        let tile = utile!(0, 0, 0);
        assert_eq!(tile, Tile::new(0, 0, 0));
    }

    #[test]
    fn test_simplify() {
        let children = utile!(243, 166, 9).children(Some(12));
        assert_eq!(children.len(), 64);
        let mut children = children.into_iter().collect::<Vec<Tile>>();
        children.truncate(61);
        children.push(children[0]);
        let simplified = simplify(children.into_iter().collect::<HashSet<Tile>>());
        let targets = vec![
            utile!(487, 332, 10),
            utile!(486, 332, 10),
            utile!(487, 333, 10),
            utile!(973, 667, 11),
            utile!(973, 666, 11),
            utile!(972, 666, 11),
            utile!(1944, 1334, 12),
        ];
        for target in targets {
            assert!(simplified.contains(&target));
        }
    }

    #[test]
    fn test_simplify_removal() {
        let tiles = vec![
            utile!(1298, 3129, 13),
            utile!(649, 1564, 12),
            utile!(650, 1564, 12),
        ];
        let simplified = simplify(tiles.into_iter().collect::<HashSet<Tile>>());
        assert!(!simplified.contains(&utile!(1298, 3129, 13)));
        assert!(simplified.contains(&utile!(650, 1564, 12)));
        assert!(simplified.contains(&utile!(649, 1564, 12)));
    }
}
