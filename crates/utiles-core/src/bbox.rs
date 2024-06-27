//! Bounding-boxes!
use crate::lnglat::LngLat;
use crate::parsing::parse_bbox;
use crate::tile::Tile;
use crate::tile_like::TileLike;
use crate::{xy, Point2d};
use serde::{Deserialize, Serialize};

/// Bounding box tuple
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct BBoxTuple(f64, f64, f64, f64);

/// Bounding box struct
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BBox {
    /// west/left boundary
    pub west: f64,
    /// south/bottom boundary
    pub south: f64,
    /// east/right boundary
    pub east: f64,
    /// north/top boundary
    pub north: f64,
}

/// Web Mercator Bounding box struct
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebBBox {
    /// lower-left corner (west, south)/(left, bottom)
    min: Point2d<f64>,

    /// upper-right corner (east, north)/(right, top)
    max: Point2d<f64>,
}

/// Bounding box containable enum
pub enum BBoxContainable {
    /// `LngLat`
    LngLat(LngLat),
    /// `BBox`
    BBox(BBox),
    /// Tile
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
    /// Create a new `BBox`
    #[must_use]
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        BBox {
            west,
            south,
            east,
            north,
        }
    }

    /// Returns a bounding box that covers the entire world.
    #[must_use]
    pub fn world_planet() -> Self {
        BBox {
            west: -180.0,
            south: -90.0,
            east: 180.0,
            north: 90.0,
        }
    }

    /// Returns a bounding box that covers the entire web mercator world.
    #[must_use]
    pub fn world_web() -> Self {
        BBox {
            west: -180.0,
            south: -85.051_129,
            east: 180.0,
            north: 85.051_129,
        }
    }

    /// Returns true if the bounding box crosses the antimeridian (the 180-degree meridian).
    #[must_use]
    pub fn crosses_antimeridian(&self) -> bool {
        self.west > self.east
    }

    /// Returns the bounding box as a tuple
    #[must_use]
    pub fn tuple(&self) -> (f64, f64, f64, f64) {
        (self.west(), self.south(), self.east(), self.north())
    }

    /// Returns the top/north boundary of the bounding box
    #[must_use]
    pub fn north(&self) -> f64 {
        self.north
    }

    /// Returns the bottom/south boundary of the bounding box
    #[must_use]
    pub fn south(&self) -> f64 {
        self.south
    }

    /// Returns the right/east boundary of the bounding box
    #[must_use]
    pub fn east(&self) -> f64 {
        self.east
    }

    /// Returns the left/west boundary of the bounding box
    #[must_use]
    pub fn west(&self) -> f64 {
        self.west
    }

    /// Returns the top/north boundary of the bounding box
    #[must_use]
    pub fn top(&self) -> f64 {
        self.north
    }

    /// Returns the bottom/south boundary of the bounding box
    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.south
    }

    /// Returns the right/east boundary of the bounding box
    #[must_use]
    pub fn right(&self) -> f64 {
        self.east
    }

    /// Returns the left/west boundary of the bounding box
    #[must_use]
    pub fn left(&self) -> f64 {
        self.west
    }

    /// Returns the geojson tuple/array representation of the bounding box
    #[must_use]
    #[inline]
    pub fn json_arr(&self) -> String {
        format!(
            "[{},{},{},{}]",
            self.west(),
            self.south(),
            self.east(),
            self.north()
        )
    }

    /// Returns the gdal-ish string representation of the bounding box
    #[must_use]
    #[inline]
    pub fn projwin_str(&self) -> String {
        format!(
            "{},{},{},{}",
            self.west(),
            self.north(),
            self.east(),
            self.south()
        )
    }

    /// Returns the center of the bounding box as a `LngLat`
    #[must_use]
    pub fn contains_lnglat(&self, lnglat: &LngLat) -> bool {
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

    /// Returns true if the current instance contains the given `Tile`
    #[must_use]
    pub fn contains_tile(&self, tile: &Tile) -> bool {
        let bbox = tile.bbox();
        self.contains_bbox(&bbox.into())
    }

    /// Returns true if the current instance contains the given `BBox`
    #[must_use]
    pub fn contains_bbox(&self, other: &BBox) -> bool {
        self.north >= other.north
            && self.south <= other.south
            && self.east >= other.east
            && self.west <= other.west
    }

    /// Returns true if the current instance contains the given `BBoxContainable` object.
    #[must_use]
    pub fn contains(&self, other: &BBoxContainable) -> bool {
        match other {
            BBoxContainable::LngLat(lnglat) => self.contains_lnglat(lnglat),
            BBoxContainable::BBox(bbox) => self.contains_bbox(bbox),
            BBoxContainable::Tile(tile) => self.contains_tile(tile),
        }
    }

    /// Returns true if the current instance is within the given bounding box.
    #[must_use]
    pub fn is_within(&self, other: &BBox) -> bool {
        self.north <= other.north
            && self.south >= other.south
            && self.east <= other.east
            && self.west >= other.west
    }

    /// Returns true if the current instance intersects with the given bounding box.
    #[must_use]
    pub fn intersects(&self, other: &BBox) -> bool {
        self.north >= other.south
            && self.south <= other.north
            && self.east >= other.west
            && self.west <= other.east
    }

    /// Returns a vector of bounding boxes (`BBox`) associated with the current instance.
    ///
    /// If the instance crosses the antimeridian (the 180-degree meridian), this function
    /// returns two `BBox` instances:
    /// - The first bounding box covers the area from the object's western boundary to 180 degrees east.
    /// - The second bounding box covers the area from -180 degrees west to the object's eastern boundary.
    ///
    /// If the instance does not cross the antimeridian, the function returns a vector
    /// containing a single `BBox` that represents the current instance itself.
    ///
    /// # Returns
    /// - `Vec<BBox>`: A vector containing one `BBox` if the instance does not cross the antimeridian,
    /// or two `BBox`es if it does.
    ///
    /// # Examples
    ///
    /// ```
    /// use utiles_core::BBox;
    /// let example = BBox::new(-10.0, -10.0, 10.0, 10.0);
    /// let bboxes = example.bboxes();
    /// assert_eq!(bboxes.len(), 1);
    ///
    /// let bboxes_crosses = BBox::new(179.0, -89.0, -179.0, 89.0).bboxes();
    /// assert_eq!(bboxes_crosses.len(), 2); // Split into two bounding boxes
    /// ```
    #[must_use]
    pub fn bboxes(&self) -> Vec<BBox> {
        if self.crosses_antimeridian() {
            vec![
                BBox {
                    north: self.north,
                    south: self.south,
                    east: 180.0,
                    west: self.west,
                },
                BBox {
                    north: self.north,
                    south: self.south,
                    east: self.east,
                    west: -180.0,
                },
            ]
        } else {
            vec![*self]
        }
    }

    /// Return upper left corner of bounding box as `LngLat`
    #[must_use]
    pub fn ul(&self) -> LngLat {
        LngLat::new(self.west, self.north)
    }

    /// Return upper right corner of bounding box as `LngLat`
    #[must_use]
    pub fn ur(&self) -> LngLat {
        LngLat::new(self.east, self.north)
    }

    /// Return lower right corner of bounding box as `LngLat`
    #[must_use]
    pub fn lr(&self) -> LngLat {
        LngLat::new(self.east, self.south)
    }

    /// Return lower left corner of bounding box as `LngLat`
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
        parse_bbox(s).unwrap_or_else(|_e| BBox::world_planet())
    }
}

impl TryFrom<&str> for BBox {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        parse_bbox(s).map_err(|_| "Failed to parse BBox")
    }
}

impl WebBBox {
    #[must_use]
    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> Self {
        WebBBox {
            min: Point2d::new(left, bottom),
            max: Point2d::new(right, top),
        }
    }

    #[must_use]
    #[inline]
    pub fn min(&self) -> Point2d<f64> {
        self.min
    }

    #[must_use]
    #[inline]
    pub fn max(&self) -> Point2d<f64> {
        self.max
    }

    #[must_use]
    #[inline]
    pub fn left(&self) -> f64 {
        self.min.x
    }

    #[must_use]
    #[inline]
    pub fn bottom(&self) -> f64 {
        self.min.y
    }

    #[must_use]
    #[inline]
    pub fn right(&self) -> f64 {
        self.max.x
    }

    #[must_use]
    #[inline]
    pub fn top(&self) -> f64 {
        self.max.y
    }

    #[must_use]
    #[inline]
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    #[must_use]
    #[inline]
    pub fn west(&self) -> f64 {
        self.min.x
    }

    #[must_use]
    #[inline]
    pub fn south(&self) -> f64 {
        self.min.y
    }

    #[must_use]
    #[inline]
    pub fn east(&self) -> f64 {
        self.max.x
    }

    #[must_use]
    #[inline]
    pub fn north(&self) -> f64 {
        self.max.y
    }

    /// Returns the geojson tuple/array representation of the bounding box
    #[must_use]
    #[inline]
    pub fn json_arr(&self) -> String {
        format!(
            "[{},{},{},{}]",
            self.west(),
            self.south(),
            self.east(),
            self.north()
        )
    }

    /// Returns the gdal-ish string representation of the bounding box
    #[must_use]
    #[inline]
    pub fn projwin_str(&self) -> String {
        format!(
            "{},{},{},{}",
            self.west(),
            self.north(),
            self.east(),
            self.south()
        )
    }
}

impl From<BBox> for WebBBox {
    fn from(bbox: BBox) -> Self {
        let (west_merc, south_merc) = xy(bbox.west(), bbox.south(), None);
        let (east_merc, north_merc) = xy(bbox.east(), bbox.north(), None);
        WebBBox::new(west_merc, south_merc, east_merc, north_merc)
    }
}

impl From<Tile> for WebBBox {
    fn from(tile: Tile) -> Self {
        let bbox = tile.geobbox();
        WebBBox::new(bbox.west, bbox.south, bbox.east, bbox.north)
    }
}
