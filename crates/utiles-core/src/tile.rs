use std::cmp::Ordering;
use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::constants::EPSILON;
use crate::errors::UtilesCoreError;
use crate::errors::UtilesCoreResult;
use crate::fns::{bounds, children, neighbors, parent, siblings, xy};
// use crate::mbutiles::MbtTileRow;
use crate::projection::Projection;
use crate::tile_feature::TileFeature;
use crate::tile_like::TileLike;
use crate::tile_tuple::TileTuple;
use crate::{pmtiles, quadkey2tile, rmid2xyz, xyz2quadkey, IsOk};

/// Tile X-Y-Z struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tile {
    /// x value (column)
    pub x: u32,

    /// y value (row -- flipped in mbtiles)
    pub y: u32,

    /// z value (zoom level)
    pub z: u8,
}

/// Geometry for tile-feature (polygon or linestring)
#[derive(Debug, Serialize, Deserialize)]
pub struct TileFeatureGeometry {
    /// type of geometry (Polygon or `LineString`)
    #[serde(rename = "type")]
    pub type_: String,

    /// coordinates for the geometry [ [ [x1, y1], [x2, y2], ... ] ]
    pub coordinates: Vec<Vec<Vec<f64>>>,
}

/// Options for creating a tile-feature
#[derive(Debug, Serialize)]
pub struct FeatureOptions {
    /// feature id to use
    pub fid: Option<String>,

    /// `GeoJSON` properties to use
    pub props: Option<Map<String, Value>>,

    /// projection to use
    pub projection: Projection,

    /// buffer size to use
    pub buffer: Option<f64>,

    /// precision to use (number of decimal places)
    pub precision: Option<i32>,
}

impl Default for FeatureOptions {
    fn default() -> Self {
        FeatureOptions {
            fid: None,
            props: None,
            projection: Projection::Geographic,
            buffer: None,
            precision: None,
        }
    }
}

impl From<TileTuple> for Tile {
    fn from(xyz: TileTuple) -> Self {
        Tile {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}y{}z{}", self.x, self.y, self.z)
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
        (self.z, self.x, self.y).cmp(&(other.z, other.x, other.y))
    }
}

impl FromStr for Tile {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // if it starts with '{' assume json obj
        if s.starts_with('{') {
            // if '{' assume its an obj
            let r = Tile::from_json_obj(s);
            return match r {
                Ok(tile) => Ok(tile),
                Err(_e) => {
                    Err(Box::from(UtilesCoreError::TileParseError(s.to_string())))
                }
            };
        } else if s.starts_with('[') {
            // if '[' assume its an arr
            let r = Tile::from_json_arr(s);
            return match r {
                Ok(tile) => Ok(tile),
                Err(_e) => {
                    Err(Box::from(UtilesCoreError::TileParseError(s.to_string())))
                }
            };
        }

        // assume its a quadkey
        let res = quadkey2tile(s);
        // if ok return tile but not tile parse error
        match res {
            Ok(tile) => Ok(tile),
            Err(_e) => Err(Box::from(UtilesCoreError::TileParseError(s.to_string()))),
        }
    }
}

impl TileLike for Tile {
    fn x(&self) -> u32 {
        self.x
    }

    fn y(&self) -> u32 {
        self.y
    }

    fn z(&self) -> u8 {
        self.z
    }
}

impl Tile {
    /// Create a new Tile
    #[must_use]
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Tile { x, y, z }
    }

    /// flip the y value (row) and return flipped tile
    #[must_use]
    pub fn flip(&self) -> Self {
        Tile::new(self.x, self.flipy(), self.z)
    }

    /// Return bounds tuple (west, south, east, north) for the tile
    #[must_use]
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        bounds(self.x, self.y, self.z)
    }

    /// Return pmtile-id for the tile
    #[must_use]
    pub fn pmtileid(&self) -> u64 {
        pmtiles::xyz2pmid(self.x, self.y, self.z)
    }

    /// Return tile from pmtile-id
    #[must_use]
    pub fn from_pmtileid(id: u64) -> Self {
        pmtiles::pmid2xyz(id).into()
    }

    /// Return tile from pmtile-id (alias for `from_pmtileid`)
    #[must_use]
    pub fn from_pmid(id: u64) -> Self {
        pmtiles::pmid2xyz(id).into()
    }

    /// Return tile from row-major tile-id
    #[must_use]
    pub fn from_row_major_id(id: u64) -> Self {
        Tile::from(rmid2xyz(id))
    }

    /// Return tile from row-major tile-id (alias for `from_row_major_id`)
    #[must_use]
    pub fn from_rmid(id: u64) -> Self {
        Tile::from_row_major_id(id)
    }

    /// Return zxy string with optional separator (default is '/')
    #[must_use]
    pub fn fmt_zxy(&self, sep: Option<&str>) -> String {
        if let Some(sep) = sep {
            format!("{}{}{}{}{}", self.z, sep, self.x, sep, self.y)
        } else {
            format!("{}/{}/{}", self.z, self.x, self.y)
        }
    }

    /// Return zxy string with extension and optional separator (default is '/')
    ///
    /// # Examples
    /// ```
    /// use utiles_core::Tile;
    /// let tile = Tile::new(1, 2, 3);
    /// assert_eq!(tile.fmt_zxy_ext("png", Some("-")), "3-1-2.png");
    /// assert_eq!(tile.fmt_zxy_ext("png", None), "3/1/2.png");
    /// ```
    #[must_use]
    pub fn fmt_zxy_ext(&self, ext: &str, sep: Option<&str>) -> String {
        if let Some(sep) = sep {
            format!("{}{}{}{}{}.{}", self.z, sep, self.x, sep, self.y, ext)
        } else {
            format!("{}/{}/{}.{}", self.z, self.x, self.y, ext)
        }
    }

    /// Return the parent tile's pmtile-id
    #[must_use]
    pub fn parent_pmtileid(&self) -> u64 {
        self.parent(None).pmtileid()
    }

    /// Convert quadkey string to Tile
    pub fn from_quadkey(quadkey: &str) -> UtilesCoreResult<Self> {
        quadkey2tile(quadkey)
    }

    /// Convert quadkey string to Tile (alias for `from_quadkey`)
    pub fn from_qk(qk: &str) -> UtilesCoreResult<Self> {
        quadkey2tile(qk)
    }

    /// Return tile from json-object string (e.g. `{"x": 1, "y": 2, "z": 3}`)
    ///
    /// # Errors
    ///
    /// Returns error if serde parsing fails
    ///
    /// # Examples
    /// ```
    /// use utiles_core::Tile;
    /// let tile = Tile::from_json_obj(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    /// assert_eq!(tile, Tile::new(1, 2, 3));
    /// ```
    pub fn from_json_obj(json: &str) -> UtilesCoreResult<Self> {
        let res = serde_json::from_str::<Tile>(json);
        match res {
            Ok(tile) => Ok(tile),
            Err(_e) => Err(UtilesCoreError::TileParseError(json.to_string())),
        }
    }

    /// Return tile from json-array string (e.g. `[1, 2, 3]`)
    ///
    /// # Errors
    ///
    /// Returns error if serde parsing fails
    ///
    /// # Examples
    /// ```
    /// use utiles_core::Tile;
    /// let tile = Tile::from_json_arr("[1, 2, 3]").unwrap();
    /// assert_eq!(tile, Tile::new(1, 2, 3));
    /// ```
    pub fn from_json_arr(json: &str) -> UtilesCoreResult<Self> {
        let res = serde_json::from_str::<(u32, u32, u8)>(json);
        match res {
            Ok((x, y, z)) => Ok(Tile::new(x, y, z)),
            Err(_e) => Err(UtilesCoreError::TileParseError(json.to_string())),
        }
    }

    /// Return tile from json string either object or array
    ///
    /// # Errors
    ///
    /// Returns error if serde parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use utiles_core::Tile;
    /// let tile_from_obj = Tile::from_json(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    /// assert_eq!(tile_from_obj, Tile::new(1, 2, 3));
    /// let tile_from_arr = Tile::from_json(r#"[1, 2, 3]"#).unwrap();
    /// assert_eq!(tile_from_arr, Tile::new(1, 2, 3));
    /// ```
    pub fn from_json(json: &str) -> Result<Self, UtilesCoreError> {
        let json_no_space = if json.starts_with(' ') {
            json.trim()
        } else {
            json
        };
        if json_no_space.starts_with('{') {
            Self::from_json_obj(json_no_space)
        } else {
            Self::from_json_arr(json_no_space)
        }
    }

    /// Return tile from json string either object or array
    ///
    /// # Errors
    ///
    /// Returns error if unable to parse json string
    pub fn from_json_loose(json: &str) -> UtilesCoreResult<Self> {
        let v = serde_json::from_str::<Value>(json)?;
        let t = Tile::try_from(&v)?;
        Ok(t)
    }

    /// Return the quadkey for the tile
    #[must_use]
    pub fn quadkey(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    /// Return the quadkey for the tile (alias for quadkey)
    #[must_use]
    pub fn qk(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    /// Return new Tile from given (lng, lat, zoom)
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails resulting in invalid tile
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_lnglat_zoom(
        lng: f64,
        lat: f64,
        zoom: u8,
        truncate: Option<bool>,
    ) -> UtilesCoreResult<Self> {
        let (x, y) = crate::_xy(lng, lat, truncate)?;
        let z2 = 2.0_f64.powi(i32::from(zoom));
        let z2f = z2;

        let xtile = if x <= 0.0 {
            0
        } else if x >= 1.0 {
            u32::try_from((z2 - 1.0).floor() as i64).unwrap_or(0)
        } else {
            let xt = (x + EPSILON) * z2f;
            u32::try_from(xt.floor() as i64).unwrap_or(0)
        };

        let ytile = if y <= 0.0 {
            0
        } else if y >= 1.0 {
            u32::try_from((z2 - 1.0).floor() as i64).unwrap_or(0)
        } else {
            let yt = (y + EPSILON) * z2f;
            u32::try_from(yt.floor() as i64).unwrap_or(0)
        };

        Ok(Self {
            x: xtile,
            y: ytile,
            z: zoom,
        })
    }

    /// Return the bounding box of the tile
    #[must_use]
    pub fn up(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    /// Return the tile to the bottom
    #[must_use]
    pub fn down(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    /// Return the tile to the left
    #[must_use]
    pub fn left(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }

    /// Return the tile to the right
    #[must_use]
    pub fn right(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    /// Return the tile to the top left
    #[must_use]
    pub fn up_left(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    /// Return the tile to the top right
    #[must_use]
    pub fn up_right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    /// Return the tile to the bottom left
    #[must_use]
    pub fn down_left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    /// Return the tile to the bottom right
    #[must_use]
    pub fn down_right(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    /// Return a vector with the 3-8 neighbors of the tile
    #[must_use]
    pub fn neighbors(&self) -> Vec<Self> {
        neighbors(self.x, self.y, self.z)
    }

    /// Return the children tiles of the tile
    #[must_use]
    pub fn children(&self, zoom: Option<u8>) -> Vec<Tile> {
        children(self.x, self.y, self.z, zoom)
    }

    /// Return the parent tile
    #[must_use]
    pub fn parent(&self, zoom: Option<u8>) -> Self {
        parent(self.x, self.y, self.z, zoom)
    }

    /// Return sibling tiles that share the same parent tile (not neighbors)
    #[must_use]
    pub fn siblings(&self) -> Vec<Self> {
        siblings(self.x, self.y, self.z)
    }

    /// Return a `TileFeature` for the tile
    pub fn feature(
        &self,
        opts: &FeatureOptions,
    ) -> Result<TileFeature, Box<dyn Error>> {
        let buffer = opts.buffer.unwrap_or(0.0);
        let precision = opts.precision.unwrap_or(-1);
        // Compute the bounds
        let (west, south, east, north) = self.bbox();
        // Handle projected coordinates
        let (mut west, mut south, mut east, mut north) = match opts.projection {
            // Projection::Geographic=> (west, south, east, north),
            Projection::Mercator => {
                // let (east_merc, north_merc) = utiles_core::xy(east, north, Some(false));
                let (west_merc, south_merc) = xy(west, south, None);
                let (east_merc, north_merc) = xy(east, north, None);
                (west_merc, south_merc, east_merc, north_merc)
            }
            Projection::Geographic => (west, south, east, north),
        };

        // Apply buffer
        west -= buffer;
        south -= buffer;
        east += buffer;
        north += buffer;

        // Apply precision
        if precision >= 0 {
            let precision_factor = 10_f64.powi(precision);
            west = (west * precision_factor).round() / precision_factor;
            south = (south * precision_factor).round() / precision_factor;
            east = (east * precision_factor).round() / precision_factor;
            north = (north * precision_factor).round() / precision_factor;
        }

        // Compute bbox and geometry
        let bbox = (
            west.min(east),
            south.min(north),
            west.max(east),
            south.max(north),
        );
        let xyz = self.tuple_string();
        let geometry_coordinates = vec![vec![
            vec![west, south],
            vec![west, north],
            vec![east, north],
            vec![east, south],
            vec![west, south],
        ]];
        let mut properties: Map<String, Value> = Map::new();
        properties.insert("title".to_string(), Value::from(format!("XYZ tile {xyz}")));
        properties.extend(opts.props.clone().unwrap_or_default());
        let id = opts.fid.clone().unwrap_or(xyz);
        let tile_feature = TileFeature {
            id,
            type_: "Feature".to_string(),
            geometry: TileFeatureGeometry {
                type_: "Polygon".to_string(),
                coordinates: geometry_coordinates,
            },
            bbox,
            properties,
        };
        Ok(tile_feature)
    }
}

impl IsOk for Tile {
    fn ok(&self) -> UtilesCoreResult<Self> {
        if self.z > 30 {
            Err(UtilesCoreError::InvalidTile(format!(
                "({},{},{}) 0 <= zoom <= 30",
                self.x, self.y, self.z
            )))
        } else {
            let z2 = 2_u32.pow(u32::from(self.z));
            if self.x >= z2 || self.y >= z2 {
                Err(UtilesCoreError::InvalidTile(format!(
                    "({},{},{}) x < 2^z and y < 2^z",
                    self.x, self.y, self.z
                )))
            } else {
                Ok(*self)
            }
        }
    }
}

impl From<(u32, u32, u8)> for Tile {
    fn from(tuple: (u32, u32, u8)) -> Self {
        TileTuple::from(tuple).into()
    }
}

impl TryFrom<&Map<String, Value>> for Tile {
    type Error = UtilesCoreError;

    fn try_from(map: &Map<String, Value>) -> Result<Self, Self::Error> {
        let x = u32::try_from(map["x"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        let y = u32::try_from(map["y"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        let z = u8::try_from(map["z"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        Ok(Tile::new(x, y, z))
    }
}

impl TryFrom<(u64, u64, u64)> for Tile {
    type Error = UtilesCoreError;

    fn try_from(tuple: (u64, u64, u64)) -> Result<Self, Self::Error> {
        let x = u32::try_from(tuple.0).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        let y = u32::try_from(tuple.1).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        let z = u8::try_from(tuple.2).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        Ok(Tile::new(x, y, z))
    }
}

impl TryFrom<&Vec<Value>> for Tile {
    type Error = UtilesCoreError;

    fn try_from(arr: &Vec<Value>) -> Result<Self, Self::Error> {
        if arr.len() < 3 {
            Err(UtilesCoreError::InvalidJson(
                serde_json::to_string(&arr).unwrap_or_default(),
            ))
        } else {
            let x = arr[0].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            let y = arr[1].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            let z = arr[2].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            Tile::try_from((x, y, z))
        }
    }
}

impl TryFrom<&Value> for Tile {
    type Error = UtilesCoreError;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::Array(v) => {
                let t = Tile::try_from(v)?;
                Ok(t)
            }
            Value::Object(v) => {
                if v.contains_key("x") && v.contains_key("y") && v.contains_key("z") {
                    let x = v["x"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    let y = v["y"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    let z = v["z"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    Tile::try_from((x, y, z))
                } else if v.contains_key("tile")
                    && v["tile"].is_array()
                    && v["tile"]
                        .as_array()
                        .expect("Unable to get tile array from Value")
                        .len()
                        == 3
                {
                    let tuple = serde_json::from_value::<TileTuple>(v["tile"].clone())?;
                    Ok(Tile::from(tuple))
                } else {
                    Err(UtilesCoreError::InvalidJson(
                        serde_json::to_string(&v)
                            .expect("Invalid json object for Tile from Value"),
                    ))
                }
            }
            _ => Err(UtilesCoreError::InvalidJson(val.to_string())),
        }
    }
}

impl TryFrom<Value> for Tile {
    type Error = UtilesCoreError;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        Tile::try_from(&val)
    }
}

// impl From<Value> for Tile {
//     fn from(val: Value) -> Self {
//         Tile::from(&val)
//     }
// }

impl TryFrom<&str> for Tile {
    type Error = UtilesCoreError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let res = Tile::from_str(s);
        match res {
            Ok(tile) => Ok(tile),
            Err(e) => Err(UtilesCoreError::TileParseError(e.to_string())),
        }
    }
}

impl From<Tile> for (u32, u32, u8) {
    fn from(tile: Tile) -> Self {
        (tile.x, tile.y, tile.z)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn parse_json_obj() {
        let json_obj = r#"{"x": 1, "y": 2, "z": 3}"#;
        let tile = Tile::from_json_obj(json_obj).unwrap();
        assert_eq!(tile, Tile::new(1, 2, 3));
    }

    #[test]
    fn parse_json_arr() {
        let json_arr = r"[1, 2, 3]";
        let tile = Tile::from_json_arr(json_arr).unwrap();
        assert_eq!(tile, Tile::new(1, 2, 3));
    }

    #[test]
    fn parse_quadkey() {
        let quadkey = "023010203";
        let tile = quadkey.parse::<Tile>();
        assert_eq!(tile.unwrap(), Tile::new(81, 197, 9));
    }

    #[test]
    fn tile_from_value_obj() {
        let json_obj = r#"{"x": 1, "y": 2, "z": 3}"#;
        let val_obj = serde_json::from_str::<Value>(json_obj).unwrap();
        let tile_from_obj = Tile::try_from(val_obj).unwrap();
        assert_eq!(tile_from_obj, Tile::new(1, 2, 3));
    }

    #[test]
    fn tile_from_value_arr() {
        let json_arr = r"[1, 2, 3]";
        let val_arr = serde_json::from_str::<Value>(json_arr).unwrap();
        let tile_from_arr = Tile::try_from(val_arr).unwrap();
        assert_eq!(tile_from_arr, Tile::new(1, 2, 3));
    }

    #[test]
    fn tile_from_value_obj_with_array() {
        let json_obj_with_tile_array = r#"{"tile": [1, 2, 3]}"#;
        let val_obj_with_tile_array =
            serde_json::from_str::<Value>(json_obj_with_tile_array).unwrap();
        let tile_from_obj_with_tile_array =
            Tile::try_from(val_obj_with_tile_array).unwrap();
        assert_eq!(tile_from_obj_with_tile_array, Tile::new(1, 2, 3));
    }
}
