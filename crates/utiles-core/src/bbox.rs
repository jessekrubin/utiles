use crate::lnglat::LngLat;
use crate::parsing::parse_bbox;
use crate::tile::Tile;
use crate::tile_like::TileLike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct BBoxTuple(f64, f64, f64, f64);

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct CoordTuple(f64, f64);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum BBoxParseAble {
    BBoxTuple((f64, f64, f64, f64)),
    CoordTuple((f64, f64)),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebMercatorBbox {
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub top: f64,
}

pub enum BBoxContainable {
    LngLat(LngLat),
    BBox(BBox),
    Tile(Tile),
}

impl From<(f64, f64, f64, f64)> for BBox {
    fn from(bbox: (f64, f64, f64, f64)) -> Self {
        BBox::new(bbox.0, bbox.1, bbox.2, bbox.3)
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

impl BBox {
    #[must_use]
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        BBox {
            north,
            south,
            east,
            west,
        }
    }

    #[must_use]
    pub fn world_planet() -> Self {
        BBox {
            west: -180.0,
            south: -90.0,
            east: 180.0,
            north: 90.0,
        }
    }

    #[must_use]
    pub fn world_web() -> Self {
        BBox {
            west: -180.0,
            south: -85.051_129,
            east: 180.0,
            north: 85.051_129,
        }
    }

    #[must_use]
    pub fn crosses_antimeridian(&self) -> bool {
        self.west > self.east
    }

    #[must_use]
    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        (self.west(), self.south(), self.east(), self.north())
    }

    #[must_use]
    pub fn north(&self) -> f64 {
        self.north
    }
    #[must_use]
    pub fn south(&self) -> f64 {
        self.south
    }
    #[must_use]
    pub fn east(&self) -> f64 {
        self.east
    }
    #[must_use]
    pub fn west(&self) -> f64 {
        self.west
    }
    #[must_use]
    pub fn top(&self) -> f64 {
        self.north
    }
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.south
    }
    #[must_use]
    pub fn right(&self) -> f64 {
        self.east
    }
    #[must_use]
    pub fn left(&self) -> f64 {
        self.west
    }

    #[must_use]
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

    #[must_use]
    pub fn contains_tile(&self, tile: Tile) -> bool {
        let bbox = tile.bbox();
        self.contains_bbox(bbox.into())
    }

    #[must_use]
    pub fn contains_bbox(&self, other: BBox) -> bool {
        self.north >= other.north
            && self.south <= other.south
            && self.east >= other.east
            && self.west <= other.west
    }

    #[must_use]
    pub fn contains(&self, other: BBoxContainable) -> bool {
        match other {
            BBoxContainable::LngLat(lnglat) => self.contains_lnglat(lnglat),
            BBoxContainable::BBox(bbox) => self.contains_bbox(bbox),
            BBoxContainable::Tile(tile) => self.contains_tile(tile),
        }
    }

    #[must_use]
    pub fn is_within(&self, other: &BBox) -> bool {
        self.north <= other.north
            && self.south >= other.south
            && self.east <= other.east
            && self.west >= other.west
    }

    #[must_use]
    pub fn intersects(&self, other: &BBox) -> bool {
        self.north >= other.south
            && self.south <= other.north
            && self.east >= other.west
            && self.west <= other.east
    }

    #[must_use]
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

    #[must_use]
    pub fn ul(&self) -> LngLat {
        LngLat::new(self.west, self.north)
    }

    #[must_use]
    pub fn ur(&self) -> LngLat {
        LngLat::new(self.east, self.north)
    }

    #[must_use]
    pub fn lr(&self) -> LngLat {
        LngLat::new(self.east, self.south)
    }

    #[must_use]
    pub fn ll(&self) -> LngLat {
        LngLat::new(self.west, self.south)
    }
}

impl From<BBox> for BBoxTuple {
    fn from(bbox: BBox) -> Self {
        BBoxTuple(bbox.west, bbox.south, bbox.east, bbox.north)
    }
}

impl From<BBoxTuple> for BBox {
    fn from(tuple: BBoxTuple) -> Self {
        BBox::new(tuple.0, tuple.1, tuple.2, tuple.3)
    }
}

impl From<&String> for BBox {
    fn from(s: &String) -> Self {
        // remove leading and trailing quotes
        let s = s.trim_matches('"');
        // let value: Value = serde_json::from_str(&s).unwrap();
        parse_bbox(s).unwrap_or_else(|_e| BBox::world_planet())
    }
}

impl From<&str> for BBox {
    fn from(s: &str) -> Self {
        parse_bbox(s).unwrap()
    }
}

impl From<String> for BBox {
    fn from(s: String) -> Self {
        self::BBox::from(&s)
    }
}

// impl From<Tile> for WebMercatorBbox {
//     fn from(tile: Tile) -> Self {
//         crate::xyz2bbox(tile.x, tile.y, tile.z)
//     }
// }
