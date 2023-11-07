use serde::de::value;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::lnglat::LngLat;
use crate::tile::Tile;

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
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
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
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        BBox {
            west: west,
            south: south,
            east: east,
            north: north,
        }
    }

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
        let tuple: BBoxTuple = serde_json::from_str(&s).unwrap();
        self::BBox::from(tuple)
    }
}

impl From<&str> for BBox {
    fn from(s: &str) -> Self {
        self::BBox::from(&s.to_string())
    }
}

impl From<String> for BBox {
    fn from(s: String) -> Self {
        self::BBox::from(&s)
    }
}

impl From<Tile> for WebMercatorBbox {
    fn from(tile: Tile) -> Self {
        crate::xyz2bbox(tile.x, tile.y, tile.z)
    }
}

//
// pub fn parse_bbox(s: &str) -> Result<BBox> {
//     let parsed : Result<BBoxParseAble>= serde_json::from_str(&s);
//     if parsed.is_err() {
//         // println!("parsed error: {:?}", parsed.err().unwrap());
//         return Err(parsed.err().unwrap())
//     }
//     let parsed = parsed.unwrap();
//     let bbox = match parsed {
//         BBoxParseAble::CoordTuple(coord) => {
//             let bbox = BBox::new(coord.0, coord.1, coord.0, coord.1);
//             bbox
//         },
//         BBoxParseAble::BBoxTuple(bbox) => {
//             let bbox = BBox::from(bbox);
//             bbox
//         },
//         // BBoxParseAble::Array(array) => {
//         //     let bbox = BBox::new(array[0], array[1], array[2], array[3]);
//         //     bbox
//         // },
//     };
//     return Ok(bbox);
//
// }

// pub fn parse_bbox(s: &str) -> Result<BBox> {
//     let parsed: Result<BBoxParseAble> = serde_json::from_str(s);
//     let bbox = match parsed? {
//         BBoxParseAble::CoordTuple(coord) => BBox::new(coord.0, coord.1, coord.0, coord.1),
//         BBoxParseAble::BBoxTuple(bbox) => BBox::from(bbox),
//         // Uncomment and handle BBoxParseAble::Array(array) if needed
//         // BBoxParseAble::Array(array) => BBox::new(array[0], array[1], array[2], array[3]),
//     };
//     Ok(bbox)
// }
pub fn parse_bbox(s: &str) -> Result<BBox> {
    let v: Value = serde_json::from_str(s)?;

    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    match v.as_array().map(|arr| arr.len()) {
        Some(2) => {
            let coord: (f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::new(coord.0, coord.1, coord.0, coord.1))
        }
        Some(4) => {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::from(bbox))
        }
        _ => Err(panic!(
            "Expected a two-element array or a four-element array"
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    //
    // #[test]
    // fn parse_bbox_simple(){
    //     let string = "[-180.0, -85, 180.0, 85]";
    //     let bbox_result = parse_bbox(string);
    //     assert!(bbox_result.is_ok());
    //     let bbox = bbox_result.unwrap();
    //     assert_eq!(bbox, BBox::new( -180.0, -85.0, 180.0, 85.0));
    //
    // }

    #[test]
    fn parse_bbox_simple() {
        let string = r#"[-180.0, -85.0, 180.0, 85.0]"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    //
    // #[test]
    // fn parse_bbox_from_coords(){
    //     let string = "[1, 2]";
    //     let bbox = parse_bbox(string).unwrap();
    //     assert_eq!(bbox, BBox::new(1.0, 2.0, 1.0, 2.0));
    // }
}
