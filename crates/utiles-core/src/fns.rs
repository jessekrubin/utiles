//! Core util(e)ity functions for working with web mercator tiles, bounding boxes, et al
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::num::FpCategory;

use crate::bbox::{BBox, WebBBox};
use crate::constants::{EARTH_CIRCUMFERENCE, EARTH_RADIUS, LL_EPSILON};
use crate::errors::UtilesCoreResult;
use crate::point2d;
use crate::sibling_relationship::SiblingRelationship;
use crate::tile_zbox::{TileZBox, TileZBoxes};
use crate::utile;
use crate::zoom::ZoomOrZooms;
use crate::Point2d;
use crate::{LngLat, Tile, UtilesCoreError};

/// Return upper left lnglat as tuple `(f64, f64)` from x,y,z
#[must_use]
pub fn ult(x: u32, y: u32, z: u8) -> (f64, f64) {
    let z2 = f64::from(2_u32.pow(u32::from(z)));
    let lon_deg = (f64::from(x) / z2) * 360.0 - 180.0;
    let lat_rad = ((1.0 - 2.0 * f64::from(y) / z2) * PI).sinh().atan();
    (lon_deg, lat_rad.to_degrees())
}

/// Return upper left lnglat as `LngLat` from x,y,z
#[must_use]
pub fn ul(x: u32, y: u32, z: u8) -> LngLat {
    let (lon_deg, lat_deg) = ult(x, y, z);
    LngLat::new(lon_deg, lat_deg)
}

/// Return lower left lnglat as `LngLat` from x,y,z
#[must_use]
pub fn ll(x: u32, y: u32, z: u8) -> LngLat {
    ul(x, y + 1, z)
}

/// Return upper right lnglat as `LngLat` from x,y,z
#[must_use]
pub fn ur(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y, z)
}

/// Return lower right lnglat as `LngLat` from x,y,z
#[must_use]
pub fn lr(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y + 1, z)
}

/// Return tuple (min, max) x/y for a zoom-level
#[must_use]
pub fn minmax(zoom: u8) -> (u32, u32) {
    (0, 2_u32.pow(u32::from(zoom)) - 1)
}

/// Return true if x, y, z is a valid tile coordinate
#[must_use]
pub fn valid(x: u32, y: u32, z: u8) -> bool {
    let (minx, maxx) = minmax(z);
    let (miny, maxy) = minmax(z);
    x >= minx && x <= maxx && y >= miny && y <= maxy
}

/// Return the inverted/y-flipped y coordinate for a given y and z
#[must_use]
#[inline]
pub fn flipy(y: u32, z: u8) -> u32 {
    2_u32.pow(u32::from(z)) - 1 - y
}

/// Return the y-flipped y coordinate for a given y and z
#[must_use]
#[inline]
pub fn yflip(y: u32, z: u8) -> u32 {
    flipy(y, z)
}

/// Return cumulative base-tile-id for a tile and the zoom level of the tile
///
/// Base-tile-id is the sum of all tiles in all zoom levels below the zoom level
/// of the tile.
#[must_use]
#[inline]
pub fn int_2_offset_zoom(i: u64) -> (u64, u8) {
    if i == 0 {
        return (0, 0);
    }
    let mut acc: u64 = 0;
    let mut z: u8 = 0;
    loop {
        let num_tiles: u64 = (1 << z) * (1 << z);
        if acc + num_tiles > i {
            return (i - acc, z);
        }
        acc += num_tiles;
        z += 1;
    }
}

/// Calculate the row-major-id for a tile which is the
/// index of the tile for the zoom level PLUS the total number of tiles
/// in all zoom levels below it for the zoom level.
///
/// (x, y, z) => x + y * 2^z + 1 + 2^z * 2^z
/// (0,0,0) is 0
/// (0,0,1) is 1
/// (0,1,1) is 2
///
/// # Examples
/// ```
/// use utiles_core::xyz2rmid;
/// let zzz = xyz2rmid(0, 0, 0);
/// assert_eq!(zzz, 0);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let xyz_0_0_1 = xyz2rmid(0, 0, 1);
/// assert_eq!(xyz_0_0_1, 1);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let xyz_0_1_1 = xyz2rmid(1, 0, 1);
/// assert_eq!(xyz_0_1_1, 2);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let xyz_1_0_1 = xyz2rmid(0, 1, 1);
/// assert_eq!(xyz_1_0_1, 3);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let xyz_1_1_1 = xyz2rmid(1, 1, 1);
/// assert_eq!(xyz_1_1_1, 4);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let one_two_three = xyz2rmid(1, 2, 3);
/// assert_eq!(one_two_three, 38);
/// ```
///
/// ```
/// use utiles_core::xyz2rmid;
/// let last_tile_in_z12 = xyz2rmid(4095, 4095, 12);
/// assert_eq!(last_tile_in_z12, 22369621 - 1); // total tiles thru z12 - 1
/// ```
#[must_use]
pub fn xyz2rmid(x: u32, y: u32, z: u8) -> u64 {
    if z == 0 {
        return u64::from(x) + u64::from(y) * 2u64.pow(u32::from(z));
    }
    let base_id: u64 = (4u64.pow(u32::from(z)) - 1) / 3;
    base_id + u64::from(x) + u64::from(y) * 2u64.pow(u32::from(z))
}

/// Calculate the xyz of the tile from a row-major-id
///
/// # Examples
/// ```
/// use utiles_core::rmid2xyz;
/// let zzz = rmid2xyz(0);
/// assert_eq!(zzz, (0, 0, 0));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let xyz_0_0_1 = rmid2xyz(1);
/// assert_eq!(xyz_0_0_1, (0, 0, 1));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let xyz_0_1_1 = rmid2xyz(2);
/// assert_eq!(xyz_0_1_1, (1, 0, 1));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let xyz_1_0_1 = rmid2xyz(3);
/// assert_eq!(xyz_1_0_1, (0, 1, 1));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let xyz_1_1_1 = rmid2xyz(4);
/// assert_eq!(xyz_1_1_1, (1, 1, 1));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let one_two_three = rmid2xyz(38);
/// assert_eq!(one_two_three, (1, 2, 3));
/// ```
///
/// ```
/// use utiles_core::rmid2xyz;
/// let last_tile_in_z12 = rmid2xyz(22369621 - 1); // total tiles thru z12 - 1
/// assert_eq!(last_tile_in_z12, (4095, 4095, 12));
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn rmid2xyz(i: u64) -> (u32, u32, u8) {
    if i == 0 {
        return (0, 0, 0);
    }
    let (i_o, z) = int_2_offset_zoom(i);
    let pow_z = 2u64.pow(u32::from(z));
    let x = i_o % pow_z;
    let y = i_o / pow_z;
    (x as u32, y as u32, z)
}

/// Calculate the zoom level for the bounding-tile of a bbox
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

/// Return the bbox tuple given x, y, z.
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

/// Truncate a longitude to the valid range of -180 to 180.
#[must_use]
pub fn truncate_lng(lng: f64) -> f64 {
    lng.clamp(-180.0, 180.0)
}

/// Truncate a latitude to the valid range of -90 to 90.
#[must_use]
pub fn truncate_lat(lat: f64) -> f64 {
    lat.clamp(-90.0, 90.0)
}

/// Truncate a `LngLat` to valid range of longitude and latitude.
#[must_use]
pub fn truncate_lnglat(lnglat: &LngLat) -> LngLat {
    LngLat::new(truncate_lng(lnglat.lng()), truncate_lat(lnglat.lat()))
}

/// Return the parent tile of a tile given x, y, z, and n (number of ancestors).
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
/// Return the children of a tile given x, y, z, and zoom in z-order.
///
/// # Examples
/// ```
/// use utiles_core::{children_zorder, utile, Tile};
/// let children = children_zorder(0, 0, 0, Some(1));
/// assert_eq!(children.len(), 4);
/// assert_eq!(children, vec![
///     utile!(0, 0, 1),
///     utile!(1, 0, 1),
///     utile!(0, 1, 1),
///     utile!(1, 1, 1),
/// ]);
/// ```
///
/// ```
/// use utiles_core::{children_zorder, utile, Tile};
/// let children = children_zorder(0, 0, 0, Some(2));
/// assert_eq!(children.len(), 16);
/// let expected = [
///     utile!(0,0,2),
///     utile!(1,0,2),
///     utile!(0,1,2),
///     utile!(1,1,2),
///     utile!(2,0,2),
///     utile!(3,0,2),
///     utile!(2,1,2),
///     utile!(3,1,2),
///     utile!(0,2,2),
///     utile!(1,2,2),
///     utile!(0,3,2),
///     utile!(1,3,2),
///     utile!(2,2,2),
///     utile!(3,2,2),
///     utile!(2,3,2),
///     utile!(3,3,2),
/// ];
/// assert_eq!(children, expected);
/// ```
///
#[must_use]
pub fn children_zorder(x: u32, y: u32, z: u8, zoom: Option<u8>) -> Vec<Tile> {
    let zoom = zoom.unwrap_or(z + 1);
    let tile = utile!(x, y, z);
    let mut tiles = vec![tile];
    while tiles[0].z < zoom {
        let (xtile, ytile, ztile) = (tiles[0].x, tiles[0].y, tiles[0].z);
        tiles.append(&mut vec![
            utile!(xtile * 2, ytile * 2, ztile + 1), // top-left
            utile!(xtile * 2 + 1, ytile * 2, ztile + 1), // top-right
            utile!(xtile * 2, ytile * 2 + 1, ztile + 1), // bottom-left
            utile!(xtile * 2 + 1, ytile * 2 + 1, ztile + 1), // bottom-right
        ]);
        tiles.remove(0);
    }
    tiles
}

/// Return the children of a tile given x, y, z, and zoom; returns children
/// in stupid `a, b, d, c` orderl; but this is the mercantile way... and I
/// am not gonna fix it right now
///
/// # Examples
/// ```
/// use utiles_core::{children, utile, Tile};
/// let children = children(0, 0, 0, Some(1));
/// assert_eq!(children.len(), 4);
/// assert_eq!(children, vec![
///     utile!(0, 0, 1),
///     utile!(1, 0, 1),
///     utile!(1, 1, 1),
///     utile!(0, 1, 1),
/// ]);
/// ```
#[must_use]
pub fn children(x: u32, y: u32, z: u8, zoom: Option<u8>) -> Vec<Tile> {
    let zoom = zoom.unwrap_or(z + 1);
    let tile = utile!(x, y, z);
    let mut tiles = vec![tile];
    while tiles[0].z < zoom {
        let (xtile, ytile, ztile) = (tiles[0].x, tiles[0].y, tiles[0].z);
        tiles.append(&mut vec![
            utile!(xtile * 2, ytile * 2, ztile + 1), // top-left
            utile!(xtile * 2 + 1, ytile * 2, ztile + 1), // top-right
            utile!(xtile * 2 + 1, ytile * 2 + 1, ztile + 1), // bottom-right
            utile!(xtile * 2, ytile * 2 + 1, ztile + 1), // bottom-left
        ]);
        tiles.remove(0);
    }
    tiles
}

/// Return the siblings of a tile given x, y, z
///
/// Siblings are tiles that share the same parent, NOT neighbors.
#[must_use]
pub fn siblings(x: u32, y: u32, z: u8) -> Vec<Tile> {
    let sibrel = SiblingRelationship::from((x, y));
    match sibrel {
        SiblingRelationship::UpperLeft => vec![
            utile!(x + 1, y, z),
            utile!(x, y + 1, z),
            utile!(x + 1, y + 1, z),
        ],
        SiblingRelationship::UpperRight => vec![
            utile!(x - 1, y, z),
            utile!(x, y + 1, z),
            utile!(x - 1, y + 1, z),
        ],
        SiblingRelationship::LowerLeft => vec![
            utile!(x + 1, y, z),
            utile!(x, y - 1, z),
            utile!(x + 1, y - 1, z),
        ],
        SiblingRelationship::LowerRight => vec![
            utile!(x - 1, y, z),
            utile!(x, y - 1, z),
            utile!(x - 1, y - 1, z),
        ],
    }
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

/// Convert lng lat to web mercator x and y
///
/// # Errors
///
/// Returns error if y can not be computed.
pub fn _xy(lng: f64, lat: f64, truncate: Option<bool>) -> UtilesCoreResult<(f64, f64)> {
    let (lng, lat) = if truncate.unwrap_or(false) {
        (truncate_lng(lng), truncate_lat(lat))
    } else {
        (lng, lat)
    };
    let sinlat = lat.to_radians().sin();
    let yish = (1.0 + sinlat) / (1.0 - sinlat);
    match yish.classify() {
        FpCategory::Infinite | FpCategory::Nan => {
            Err(UtilesCoreError::LngLat2WebMercator(
                "Y can not be computed: lat={lat}".to_string(),
            ))
        }
        _ => {
            let y = 0.5 - 0.25 * (yish.ln()) / PI;
            let x = lng / 360.0 + 0.5;
            Ok((x, y))
        }
    }
}

/// Convert lng lat to web mercator x and y
#[must_use]
pub fn lnglat2webmercator(lng: f64, lat: f64) -> (f64, f64) {
    let x = EARTH_RADIUS * lng.to_radians();
    let y = if (lat - 90.0).abs() < f64::EPSILON {
        f64::INFINITY
    } else if (lat + 90.0).abs() < f64::EPSILON {
        f64::NEG_INFINITY
    } else {
        EARTH_RADIUS * (PI * 0.25 + 0.5 * lat.to_radians()).tan().ln()
    };
    (x, y)
}

/// Convert web mercator x and y to longitude and latitude.
///
/// # Examples
/// ```
/// use utiles_core::webmercator2lnglat;
/// let (lng, lat) = webmercator2lnglat(0.5, 0.5);
/// assert!((lng - 0.0).abs() < 0.0001, "lng: {}", lng);
/// assert!((lat - 0.0).abs() < 0.0001, "lat: {}", lat);
/// ```
///
#[must_use]
#[inline]
pub fn webmercator2lnglat(x: f64, y: f64) -> (f64, f64) {
    let lng = x / EARTH_RADIUS * 180.0 / PI;
    let lat = (2.0 * (y / EARTH_RADIUS).exp().atan() - PI * 0.5) * 180.0 / PI;
    (lng, lat)
}

/// Convert longitude and latitude to web mercator x and y with optional truncation.
///
/// Name "xy" comes from mercantile python library.
#[must_use]
pub fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    let (lng, lat) = if truncate.unwrap_or(false) {
        (truncate_lng(lng), truncate_lat(lat))
    } else {
        (lng, lat)
    };
    lnglat2webmercator(lng, lat)
}

/// Convert web mercator x and y to longitude and latitude with optional truncation.
#[must_use]
pub fn lnglat(x: f64, y: f64, truncate: Option<bool>) -> LngLat {
    let (lng, lat) = webmercator2lnglat(x, y);
    if truncate.is_some() {
        truncate_lnglat(&LngLat::new(lng, lat))
    } else {
        LngLat::new(lng, lat)
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
    if x == max_xy && y == max_xy {
        return TileEdgeInfo::BottomRight;
    }
    match (x, y) {
        (max, 0) if max == max_xy => TileEdgeInfo::TopRight,
        (0, max) if max == max_xy => TileEdgeInfo::BottomLeft,
        (0, _) => TileEdgeInfo::Left,
        (max, _) if max == max_xy => TileEdgeInfo::Right,
        (_, 0) => TileEdgeInfo::Top,
        (_, max) if max == max_xy => TileEdgeInfo::Bottom,
        _ => TileEdgeInfo::Middle,
    }
}

fn _neighbors_middle_tile(x: u32, y: u32, z: u8) -> Vec<Tile> {
    vec![
        utile!(x + 1, y, z),
        utile!(x, y + 1, z),
        utile!(x + 1, y + 1, z),
        utile!(x - 1, y, z),
        utile!(x, y - 1, z),
        utile!(x - 1, y - 1, z),
        utile!(x + 1, y - 1, z),
        utile!(x - 1, y + 1, z),
    ]
}

/// Return neighbors of a tile (non-wrapping).
#[must_use]
pub fn neighbors(x: u32, y: u32, z: u8) -> Vec<Tile> {
    if z == 0 {
        return Vec::new();
    }
    let edge_info = _tile_edge_info(x, y, z);
    match edge_info {
        TileEdgeInfo::Middle => _neighbors_middle_tile(x, y, z),
        TileEdgeInfo::TopLeft => vec![
            utile!(x + 1, y, z),
            utile!(x, y + 1, z),
            utile!(x + 1, y + 1, z),
        ],
        TileEdgeInfo::TopRight => vec![
            utile!(x - 1, y, z),
            utile!(x, y + 1, z),
            utile!(x - 1, y + 1, z),
        ],
        TileEdgeInfo::BottomLeft => vec![
            utile!(x + 1, y, z),
            utile!(x, y - 1, z),
            utile!(x + 1, y - 1, z),
        ],
        TileEdgeInfo::BottomRight => vec![
            utile!(x - 1, y, z),
            utile!(x, y - 1, z),
            utile!(x - 1, y - 1, z),
        ],
        TileEdgeInfo::Left => vec![
            utile!(x + 1, y, z),
            utile!(x, y + 1, z),
            utile!(x + 1, y + 1, z),
            utile!(x, y - 1, z),
            utile!(x + 1, y - 1, z),
        ],
        TileEdgeInfo::Right => vec![
            utile!(x - 1, y, z),
            utile!(x, y + 1, z),
            utile!(x - 1, y + 1, z),
            utile!(x, y - 1, z),
            utile!(x - 1, y - 1, z),
        ],
        TileEdgeInfo::Top => vec![
            utile!(x + 1, y, z),
            utile!(x, y + 1, z),
            utile!(x + 1, y + 1, z),
            utile!(x - 1, y, z),
            utile!(x - 1, y + 1, z),
        ],
        TileEdgeInfo::Bottom => vec![
            utile!(x + 1, y, z),
            utile!(x, y - 1, z),
            utile!(x + 1, y - 1, z),
            utile!(x - 1, y, z),
            utile!(x - 1, y - 1, z),
        ],
    }
}

/// Return Tile struct from longitude, latitude, and zoom.
pub fn tile(
    lng: f64,
    lat: f64,
    zoom: u8,
    truncate: Option<bool>,
) -> Result<Tile, UtilesCoreError> {
    Tile::from_lnglat_zoom(lng, lat, zoom, truncate)
}

/// Return the bounding tile for a bounding box.
pub fn bounding_tile(
    bbox: BBox,
    truncate: Option<bool>,
) -> Result<Tile, UtilesCoreError> {
    let (west, south, east, north) =
        bbox_truncate(bbox.west, bbox.south, bbox.east, bbox.north, truncate);
    let tmin = tile(west, north, 32, truncate)?;
    let tmax = tile(east - LL_EPSILON, south + LL_EPSILON, 32, truncate)?;

    let cell = (tmin.x, tmin.y, tmax.x, tmax.y);
    let z = bbox2zoom(cell);
    if z == 0 {
        Ok(utile!(0, 0, 0))
    } else {
        let x = cell.0 >> (32 - z);
        let y = cell.1 >> (32 - z);
        Ok(utile!(x, y, z))
    }
}

/// Return web-mercator bbox from x, y, z.
#[must_use]
pub fn xyz2bbox(x: u32, y: u32, z: u8) -> WebBBox {
    let tile_size = EARTH_CIRCUMFERENCE / 2.0_f64.powi(i32::from(z));
    let left = f64::from(x) * tile_size - EARTH_CIRCUMFERENCE / 2.0;
    let top = EARTH_CIRCUMFERENCE / 2.0 - f64::from(y) * tile_size;

    WebBBox::new(left, top - tile_size, left + tile_size, top)
}

/// Return zooms-vec from a `ZoomOrZooms` enum
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

/// Return `TileRanges` from a bounding box and zoom(s).
pub fn tile_ranges(
    bounds: (f64, f64, f64, f64),
    zooms: ZoomOrZooms,
) -> Result<TileZBoxes, UtilesCoreError> {
    let zooms = as_zooms(zooms);
    let bboxes: Vec<BBox> = BBox::from(bounds)
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
    let ranges: Vec<TileZBox> = bboxes
        .into_iter()
        .flat_map(move |bbox| {
            let zooms = zooms.clone();
            zooms.into_iter().map(move |zoom| {
                let upper_left_lnglat = LngLat {
                    xy: point2d! { x: bbox.west, y: bbox.north },
                };
                let lower_right_lnglat = LngLat {
                    xy: point2d! { x: bbox.east, y: bbox.south },
                };
                let top_left_tile = Tile::from_lnglat_zoom(
                    upper_left_lnglat.lng(),
                    upper_left_lnglat.lat(),
                    zoom,
                    Some(false),
                )?;
                let bottom_right_tile = Tile::from_lnglat_zoom(
                    lower_right_lnglat.lng() - LL_EPSILON,
                    lower_right_lnglat.lat() + LL_EPSILON,
                    zoom,
                    Some(false),
                )?;
                Ok(TileZBox::new(
                    top_left_tile.x,
                    bottom_right_tile.x,
                    top_left_tile.y,
                    bottom_right_tile.y,
                    zoom,
                ))
            })
        })
        .collect::<Result<Vec<TileZBox>, UtilesCoreError>>()?;
    Ok(TileZBoxes::from(ranges))
}

/// Return the number of tiles for a bounding box and zoom(s).
pub fn tiles_count(
    bounds: (f64, f64, f64, f64),
    zooms: ZoomOrZooms,
) -> Result<u64, UtilesCoreError> {
    let ranges = tile_ranges(bounds, zooms)?;
    Ok(ranges.length())
}

/// Return an iterator of tiles for a bounding box and zoom(s).
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
                xy: point2d! { x: bbox.west, y: bbox.north },
            };
            let lower_right_lnglat = LngLat {
                xy: point2d! { x: bbox.east, y: bbox.south },
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

            match (top_left_tile, bottom_right_tile) {
                (Ok(top_left), Ok(bottom_right)) => tiles_range_zoom(
                    top_left.x,
                    bottom_right.x,
                    top_left.y,
                    bottom_right.y,
                    zoom,
                )
                .map(move |(x, y, z)| Tile { x, y, z })
                .collect::<Vec<_>>()
                .into_iter(),
                _ => Vec::new().into_iter(),
            }
        })
    })
}

#[allow(dead_code)]
fn merge(merge_set: &HashSet<Tile>) -> (HashSet<Tile>, bool) {
    let mut upwards_merge: HashMap<Tile, HashSet<Tile>> = HashMap::new();
    for tile in merge_set {
        let tile_parent = tile.parent(None);
        let children_set = upwards_merge.entry(tile_parent).or_default();
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

/// Simplify a set of tiles replacing children with parents where possible.
#[allow(dead_code)]
#[must_use]
pub fn simplify<S: ::std::hash::BuildHasher + Default>(
    tiles: HashSet<Tile, S>,
) -> HashSet<Tile> {
    // pub fn simplify(tiles: HashSet<Tile>) -> HashSet<Tile> {
    // Parse tiles from the input sequence
    let mut tilesv = tiles.into_iter().collect::<Vec<Tile>>();

    tilesv.sort_by_key(|t| t.z);
    // Check to see if a tile and its parent both already exist.
    // Ensure that tiles are sorted by zoom so parents are encountered first.
    // If so, discard the child (it's covered in the parent)
    // let mut root_set: HashSet<Tile> = HashSet::new();
    let mut root_set: HashSet<Tile> = HashSet::default();
    for tile in &tilesv {
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
