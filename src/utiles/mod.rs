use constants::{EARTH_CIRCUMFERENCE, EARTH_RADIUS, EPSILON, LL_EPSILON};
use geo_types::{coord, Coord};
use serde::{Deserialize, Serialize};
use sibling_relationship::SiblingRelationship;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::num::FpCategory;
use std::{error::Error, f64::consts::PI};
use zoom::ZoomOrZooms;
mod constants;
pub mod libtiletype;
mod pmtiles;
mod sibling_relationship;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub struct XYZ(pub u32, pub u32, pub u8);

impl traits::Utiles<LngLat, BBox> for Tile {
    fn ul(&self) -> LngLat {
        ul(self.x, self.y, self.z)
    }

    fn ur(&self) -> LngLat {
        ur(self.x, self.y, self.z)
    }

    fn lr(&self) -> LngLat {
        lr(self.x, self.y, self.z)
    }

    fn ll(&self) -> LngLat {
        ll(self.x, self.y, self.z)
    }

    fn bbox(&self) -> BBox {
        let (west, south, east, north) = bounds(self.x, self.y, self.z);
        BBox {
            north,
            south,
            east,
            west,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LngLat {
    pub xy: Coord,
}

impl From<Coord> for LngLat {
    fn from(xy: Coord) -> Self {
        LngLat::new(xy.x, xy.y)
    }
}

impl From<(f64, f64)> for LngLat {
    fn from(xy: (f64, f64)) -> Self {
        LngLat::new(xy.0, xy.1)
    }
}

impl LngLat {
    pub fn new(lng: f64, lat: f64) -> Self {
        LngLat {
            xy: coord! { x: lng, y: lat},
        }
    }

    pub fn lng(&self) -> f64 {
        self.xy.x
    }

    pub fn lat(&self) -> f64 {
        self.xy.y
    }

    pub fn lon(&self) -> f64 {
        self.xy.x
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

impl From<XYZ> for Tile {
    fn from(xyz: XYZ) -> Self {
        Tile {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl From<(f64, f64, f64, f64)> for BBox {
    fn from(bbox: (f64, f64, f64, f64)) -> Self {
        BBox {
            north: bbox.0,
            south: bbox.1,
            east: bbox.2,
            west: bbox.3,
        }
    }
}

impl From<(i32, i32, i32, i32)> for BBox {
    fn from(bbox: (i32, i32, i32, i32)) -> Self {
        // convert to f64
        let bbox = (
            f64::from(bbox.0),
            f64::from(bbox.1),
            f64::from(bbox.2),
            f64::from(bbox.3),
        );
        BBox {
            north: bbox.0,
            south: bbox.1,
            east: bbox.2,
            west: bbox.3,
        }
    }
}

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

pub enum BBoxContainable {
    LngLat(LngLat),
    BBox(BBox),
    Tile(Tile),
}

impl BBox {
    pub fn crosses_antimeridian(&self) -> bool {
        self.west > self.east
    }

    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        (self.north, self.south, self.east, self.west)
    }

    pub fn north(&self) -> f64 {
        self.north
    }
    pub fn south(&self) -> f64 {
        self.south
    }
    pub fn east(&self) -> f64 {
        self.east
    }
    pub fn west(&self) -> f64 {
        self.west
    }
    pub fn top(&self) -> f64 {
        self.north
    }
    pub fn bottom(&self) -> f64 {
        self.south
    }
    pub fn right(&self) -> f64 {
        self.east
    }
    pub fn left(&self) -> f64 {
        self.west
    }

    pub fn contains_lnglat(&self, lnglat: LngLat) -> bool {
        let lng = lnglat.lng();
        let lat = lnglat.lat();
        if self.crosses_antimeridian() {
            if (lng >= self.west || lng <= self.east)
                && lat >= self.south
                && lat <= self.north
            {
                return true;
            }
        } else if lng >= self.west
            && lng <= self.east
            && lat >= self.south
            && lat <= self.north
        {
            return true;
        }
        false
    }

    pub fn contains_tile(&self, tile: Tile) -> bool {
        let bbox = tile.bbox();
        self.contains_bbox(bbox.into())
    }

    pub fn contains_bbox(&self, other: BBox) -> bool {
        self.north >= other.north
            && self.south <= other.south
            && self.east >= other.east
            && self.west <= other.west
    }

    pub fn contains(&self, other: BBoxContainable) -> bool {
        match other {
            BBoxContainable::LngLat(lnglat) => self.contains_lnglat(lnglat),
            BBoxContainable::BBox(bbox) => self.contains_bbox(bbox),
            BBoxContainable::Tile(tile) => self.contains_tile(tile),
        }
    }

    pub fn is_within(&self, other: &BBox) -> bool {
        self.north <= other.north
            && self.south >= other.south
            && self.east <= other.east
            && self.west >= other.west
    }

    pub fn intersects(&self, other: &BBox) -> bool {
        self.north >= other.south
            && self.south <= other.north
            && self.east >= other.west
            && self.west <= other.east
    }

    pub fn bboxes(&self) -> Vec<BBox> {
        if self.crosses_antimeridian() {
            let mut bboxes = Vec::new();
            let bbox1 = BBox {
                north: self.north,
                south: self.south,
                east: 180.0,
                west: self.west,
            };
            let bbox2 = BBox {
                north: self.north,
                south: self.south,
                east: self.east,
                west: -180.0,
            };
            bboxes.push(bbox1);
            bboxes.push(bbox2);
            bboxes
        } else {
            vec![*self]
        }
    }

    pub fn ul(&self) -> LngLat {
        LngLat::new(self.west, self.north)
    }

    pub fn ur(&self) -> LngLat {
        LngLat::new(self.east, self.north)
    }

    pub fn lr(&self) -> LngLat {
        LngLat::new(self.east, self.south)
    }

    pub fn ll(&self) -> LngLat {
        LngLat::new(self.west, self.south)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebMercatorBbox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

pub fn minmax(zoom: u32) -> (u32, u32) {
    (0, 2_u32.pow(zoom) - 1)
}

pub fn valid(x: u32, y: u32, z: u8) -> bool {
    let (minx, maxx) = minmax(u32::from(z));
    let (miny, maxy) = minmax(u32::from(z));
    x >= minx && x <= maxx && y >= miny && y <= maxy
}

pub fn flipy(y: u32, z: u8) -> u32 {
    2_u32.pow(u32::from(z)) - 1 - y
}

pub fn get_bbox_zoom(bbox: (u32, u32, u32, u32)) -> u8 {
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

pub fn ul(x: u32, y: u32, z: u8) -> LngLat {
    let (lon_deg, lat_deg) = ult(x, y, z);
    LngLat {
        xy: coord! {x: lon_deg, y: lat_deg},
    }
}

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

pub fn truncate_lng(lng: f64) -> f64 {
    if lng > 180.0 {
        180.0
    } else if lng < -180.0 {
        -180.0
    } else {
        lng
    }
}

pub fn truncate_lat(lat: f64) -> f64 {
    if lat > 90.0 {
        90.0
    } else if lat < -90.0 {
        -90.0
    } else {
        lat
    }
}

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
pub fn _xy_og(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    let trunc = truncate.unwrap_or(false);

    let mut lng = lng;
    let mut lat = lat;
    if trunc {
        lng = truncate_lng(lng);
        lat = truncate_lat(lat);
    }
    let x = (lng / 360.0) + 0.5;

    let sinlat = lat.to_radians().sin();

    let y_inner = (1.0 + sinlat) / (1.0 - sinlat);
    let y = match (1.0 + sinlat) / (1.0 - sinlat) {
        y if y.is_infinite() => {
            panic!("Invalid latitude (inf): {lat:?}");
        }
        y if y.is_nan() => {
            panic!("Invalid latitude (nan): {lat:?}");
        }
        // y => 0.5 - 0.25 * y.ln() / std::f64::consts::PI,
        // y = 0.5 - 0.25 * math.log((1.0 + sinlat) / (1.0 - sinlat)) / math.pi
        _y => 0.5 - 0.25 * y_inner.ln() / PI,
    };
    (x, y)
    // y = 0.5 - 0.25 * math.log((1.0 + sinlat) / (1.0 - sinlat)) / math.pi
}

pub fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    let trunc = truncate.unwrap_or(false);
    let mut lng = lng;
    let mut lat = lat;
    if trunc {
        lng = truncate_lng(lng);
        lat = truncate_lat(lat);
    }
    let x = EARTH_RADIUS * lng.to_radians();

    // let y = match (1.0 + sinlat) / (1.0 - sinlat) {
    //     y if y.is_infinite() => {
    //         panic!("Invalid latitude: {:?}", lat);
    //     }
    //     y if y.is_nan() => {
    //         panic!("Invalid latitude: {:?}", lat);
    //     }
    //     y => 0.5 - 0.25 * y.ln() / std::f64::consts::PI,
    // };
    // y = 0.5 - 0.25 * math.log((1.0 + sinlat) / (1.0 - sinlat)) / math.pi
    // let y = 0.5 - 0.25 * ((1.0 + sinlat) / (1.0 - sinlat)).ln() / PI;
    let y = if lat == 90.0 {
        f64::INFINITY
    } else if lat == -90.0 {
        f64::NEG_INFINITY
    } else {
        // (1.0 + (lat.to_radians()).sin()) / (1.0 - (lat.to_radians()).sin())
        EARTH_RADIUS * (PI * 0.25 + 0.5 * lat.to_radians()).tan().ln()
    };

    // let y = EARTH_RADIUS * (PI * 0.25 + 0.5 * lat.to_radians()).tan().ln();
    // let x  = (lnglat.lng + 180.0) / 360.0;
    // let sin_lat = lnglat.lat.to_radians().sin();
    // let y = 0.5 - (0.5 * (1.0 + sin_lat) / (1.0 - sin_lat)).ln() / PI;
    (x, y)
}

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

pub fn ll(x: u32, y: u32, z: u8) -> LngLat {
    ul(x, y + 1, z)
}

pub fn ur(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y, z)
}

pub fn lr(x: u32, y: u32, z: u8) -> LngLat {
    ul(x + 1, y + 1, z)
}

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

// tile = reptiles.Tile(486, 332, 10)
// expected = "0313102310"
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
        // write!(quadkey, "{}", digit).unwrap();
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

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}y{}z{}", self.x, self.y, self.z)
    }
}

impl std::fmt::Display for LngLat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.xy.x, self.xy.y)
    }
}

impl PartialOrd<Self> for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.z != other.z {
            return self.z.cmp(&other.z);
        }
        if self.y != other.y {
            return self.y.cmp(&other.y);
        }
        self.x.cmp(&other.x)
    }
}

impl Tile {
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Tile { x, y, z }
    }

    #[allow(dead_code)]
    pub fn valid(&self) -> bool {
        valid(self.x, self.y, self.z)
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn z(&self) -> u8 {
        self.z
    }

    pub fn zoom(&self) -> u8 {
        self.z
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        bounds(self.x, self.y, self.z)
    }

    pub fn pmtileid(&self) -> u64 {
        pmtiles::xyz2id(self.x, self.y, self.z)
    }

    pub fn from_pmtileid(id: u64) -> Self {
        let (x, y, z) = pmtiles::id2xyz(id);
        Tile::new(x, y, z)
    }

    pub fn fmt_zxy(&self) -> String {
        format!("{}/{}/{}", self.z, self.x, self.y)
    }

    pub fn fmt_zxy_ext(&self, ext: &str) -> String {
        format!("{}/{}/{}.{}", self.z, self.x, self.y, ext)
    }

    pub fn parent_id(&self) -> u64 {
        pmtiles::parent_id(self.pmtileid())
    }

    pub fn from_quadkey(quadkey: &str) -> Result<Tile, Box<dyn Error>> {
        quadkey2tile(quadkey)
    }

    pub fn from_qk(qk: &str) -> Self {
        let res = quadkey2tile(qk);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid quadkey: {e}");
            }
        }
    }

    pub fn quadkey(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    pub fn qk(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    pub fn from_lnglat_zoom(
        lng: f64,
        lat: f64,
        zoom: u8,
        truncate: Option<bool>,
    ) -> Self {
        let xy = _xy(lng, lat, truncate);
        let (x, y) = match xy {
            Ok(xy) => xy,
            Err(e) => {
                panic!("Invalid lnglat: {e}");
            }
        };
        let z2 = 2.0_f64.powi(i32::from(zoom));
        let z2f = z2;
        let xtile = if x <= 0.0 {
            0
        } else if x >= 1.0 {
            (z2f - 1.0) as u32
        } else {
            let xt = (x + EPSILON) * z2f;
            (xt.floor()) as u32
        };

        let ytile = if y <= 0.0 {
            0
        } else if y >= 1.0 {
            (z2f - 1.0) as u32
        } else {
            let yt = (y + EPSILON) * z2f;
            (yt.floor()) as u32
        };
        Self {
            x: xtile,
            y: ytile,
            z: zoom,
        }
    }

    pub fn ul(&self) -> LngLat {
        ul(self.x, self.y, self.z)
    }

    pub fn ll(&self) -> LngLat {
        ll(self.x, self.y, self.z)
    }

    pub fn ur(&self) -> LngLat {
        ur(self.x, self.y, self.z)
    }

    pub fn lr(&self) -> LngLat {
        lr(self.x, self.y, self.z)
    }

    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        let ul = self.ul();
        let lr = self.lr();
        (ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    pub fn center(&self) -> LngLat {
        let ul = self.ul();
        let lr = self.lr();
        LngLat::new((ul.lng() + lr.lng()) / 2.0, (ul.lat() + lr.lat()) / 2.0)
    }

    pub fn up(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn down(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn left(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn right(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn up_left(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn up_right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn down_left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    pub fn down_right(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn neighbors(&self) -> Vec<Self> {
        neighbors(self.x, self.y, self.z)
    }

    pub fn children(&self, zoom: Option<u8>) -> Vec<Tile> {
        children(self.x, self.y, self.z, zoom)
    }

    pub fn parent(&self, zoom: Option<u8>) -> Self {
        parent(self.x, self.y, self.z, zoom)
    }

    pub fn siblings(&self) -> Vec<Self> {
        siblings(self.x, self.y, self.z)
    }
}

impl From<Tile> for (u32, u32, u8) {
    fn from(tile: Tile) -> Self {
        (tile.x, tile.y, tile.z)
    }
}

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

impl From<Tile> for WebMercatorBbox {
    fn from(tile: Tile) -> Self {
        xyz2bbox(tile.x, tile.y, tile.z)
    }
}

pub fn as_zooms(zoom_or_zooms: ZoomOrZooms) -> Vec<u8> {
    match zoom_or_zooms {
        ZoomOrZooms::Zoom(zoom) => {
            vec![zoom]
        }
        ZoomOrZooms::Zooms(zooms) => zooms,
    }
}

pub struct TilesRange {
    curx: u32,
    cury: u32,
    pub minx: u32,
    pub maxx: u32,
    pub miny: u32,
    pub maxy: u32,
    pub zoom: u8,
}

impl Iterator for TilesRange {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curx > self.maxx {
            self.curx = self.minx;
            self.cury += 1;
        }
        if self.cury > self.maxy {
            return None;
        }
        let tile = (self.curx, self.cury, self.zoom);
        self.curx += 1;
        Some(tile)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = ((self.maxx - self.minx + 1) * (self.maxy - self.miny + 1)) as usize;
        (size, Some(size))
    }
}

pub struct TilesRanges {
    ranges: Vec<TilesRange>,
}

impl Iterator for TilesRanges {
    type Item = (u32, u32, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ranges.is_empty() {
            return None;
        }
        let mut range = self.ranges.remove(0);
        let tile = range.next();
        if let Some((_, _, _)) = tile {
            self.ranges.push(range);
        }
        tile
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

pub fn tile(lng: f64, lat: f64, zoom: u8, truncate: Option<bool>) -> Tile {
    Tile::from_lnglat_zoom(lng, lat, zoom, truncate)
}

pub fn bounding_tile(bbox: BBox, truncate: Option<bool>) -> Tile {
    let (west, south, east, north) =
        bbox_truncate(bbox.west, bbox.south, bbox.east, bbox.north, truncate);
    let tmin = tile(west, north, 32, truncate);
    let tmax = tile(east - LL_EPSILON, south + LL_EPSILON, 32, truncate);

    let cell = (tmin.x, tmin.y, tmax.x, tmax.y);
    let z = get_bbox_zoom(cell);
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
