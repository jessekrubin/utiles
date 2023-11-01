use crate::lnglat::LngLat;
use crate::tile::Tile;
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

impl From<Tile> for WebMercatorBbox {
    fn from(tile: Tile) -> Self {
        crate::xyz2bbox(tile.x, tile.y, tile.z)
    }
}
