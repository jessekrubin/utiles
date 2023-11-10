use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::error::Error;
use std::str::FromStr;

use crate::utile;

use crate::bbox::BBox;
use crate::constants::EPSILON;
use crate::lnglat::LngLat;
use crate::projection::Projection;
use crate::tile_feature::TileFeature;
use crate::tile_tuple::XYZ;
use crate::{
    bounds, children, flipy, ll, lr, neighbors, parent, pmtiles, quadkey2tile,
    siblings, traits, ul, ur, xy, xyz2quadkey,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileFeatureGeometry {
    #[serde(rename = "type")]
    pub type_: String,
    pub coordinates: Vec<Vec<Vec<f64>>>,
}

#[derive(Debug, Serialize)]
pub struct FeatureOptions {
    pub fid: Option<String>, // feature id
    pub props: Option<Map<String, Value>>,
    pub projection: Projection,
    pub buffer: Option<f64>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

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

impl From<XYZ> for Tile {
    fn from(xyz: XYZ) -> Self {
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
        if self.z != other.z {
            return self.z.cmp(&other.z);
        }
        if self.y != other.y {
            return self.y.cmp(&other.y);
        }
        self.x.cmp(&other.x)
    }
}

impl FromStr for Tile {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // if it starts with '{' assume json obj
        if s.starts_with('{') {
            // if '{' assume its an obj
            return Ok(Tile::from_json_obj(s));
        } else if s.starts_with('[') {
            return Ok(Tile::from_json_arr(s));
        }
        // assume its a quadkey
        let res = quadkey2tile(s);
        match res {
            Ok(tile) => Ok(tile),
            Err(e) => {
                panic!("Invalid quadkey: {e}");
            }
        }
    }
}

impl Tile {
    #[must_use]
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Tile { x, y, z }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn valid(&self) -> bool {
        crate::valid(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[must_use]
    pub fn z(&self) -> u8 {
        self.z
    }

    #[must_use]
    pub fn zoom(&self) -> u8 {
        self.z
    }

    #[must_use]
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        bounds(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn pmtileid(&self) -> u64 {
        pmtiles::xyz2pmid(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn from_pmtileid(id: u64) -> Self {
        let (x, y, z) = pmtiles::pmid2xyz(id);
        Tile::new(x, y, z)
    }

    #[must_use]
    pub fn fmt_zxy(&self, sep: Option<&str>) -> String {
        match sep {
            Some(sep) => format!("{}{}{}{}{}", self.z, sep, self.x, sep, self.y),
            None => format!("{}/{}/{}", self.z, self.x, self.y),
        }
    }

    #[must_use]
    pub fn fmt_zxy_ext(&self, ext: &str, sep: Option<&str>) -> String {
        match sep {
            Some(sep) => {
                format!("{}{}{}{}{}.{}", self.z, sep, self.x, sep, self.y, ext)
            }
            None => format!("{}/{}/{}.{}", self.z, self.x, self.y, ext),
        }
    }

    #[must_use]
    pub fn parent_id(&self) -> u64 {
        pmtiles::parent_id(self.pmtileid())
    }

    pub fn from_quadkey(quadkey: &str) -> Result<Tile, Box<dyn Error>> {
        quadkey2tile(quadkey)
    }

    #[must_use]
    pub fn from_qk(qk: &str) -> Self {
        let res = quadkey2tile(qk);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid quadkey: {e}");
            }
        }
    }

    #[must_use]
    pub fn from_json_obj(json: &str) -> Self {
        let res = serde_json::from_str(json);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid json_arr: {e}");
            }
        }
    }

    #[must_use]
    pub fn from_json_arr(json: &str) -> Self {
        let res = serde_json::from_str(json);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid json_arr: {e}");
            }
        }
    }

    #[must_use]
    pub fn from_json(json: &str) -> Self {
        if json.starts_with('[') {
            return Self::from_json_arr(json);
        }
        Self::from_json_obj(json)
    }

    #[must_use]
    pub fn from_json_loose(json: &str) -> Self {
        let v = serde_json::from_str::<Value>(json).unwrap();
        Self::from(v)
    }

    #[must_use]
    pub fn quadkey(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn qk(&self) -> String {
        xyz2quadkey(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn from_lnglat_zoom(
        lng: f64,
        lat: f64,
        zoom: u8,
        truncate: Option<bool>,
    ) -> Self {
        let xy = crate::_xy(lng, lat, truncate);
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

    #[must_use]
    pub fn ul(&self) -> LngLat {
        ul(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn ll(&self) -> LngLat {
        ll(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn ur(&self) -> LngLat {
        ur(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn lr(&self) -> LngLat {
        lr(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        let ul = self.ul();
        let lr = self.lr();
        (ul.lng(), lr.lat(), lr.lng(), ul.lat())
    }

    #[must_use]
    pub fn center(&self) -> LngLat {
        let ul = self.ul();
        let lr = self.lr();
        LngLat::new((ul.lng() + lr.lng()) / 2.0, (ul.lat() + lr.lat()) / 2.0)
    }

    #[must_use]
    pub fn up(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    #[must_use]
    pub fn down(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    #[must_use]
    pub fn left(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn right(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn up_left(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn up_right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn down_left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y - 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn down_right(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y + 1,
            z: self.z,
        }
    }

    #[must_use]
    pub fn neighbors(&self) -> Vec<Self> {
        neighbors(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn children(&self, zoom: Option<u8>) -> Vec<Tile> {
        children(self.x, self.y, self.z, zoom)
    }

    #[must_use]
    pub fn parent(&self, zoom: Option<u8>) -> Self {
        parent(self.x, self.y, self.z, zoom)
    }

    #[must_use]
    pub fn siblings(&self) -> Vec<Self> {
        siblings(self.x, self.y, self.z)
    }

    #[must_use]
    pub fn sql_where(&self, flip: Option<bool>) -> String {
        // classic mbtiles sqlite query:
        // 'SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?',

        // flip y for tms (default for mbtiles)
        match flip.unwrap_or(true) {
            true => format!(
                "(zoom_level = {} AND tile_column = {} AND tile_row = {})",
                self.z,
                self.x,
                flipy(self.y, self.z)
            ),
            false => format!(
                "(zoom_level = {} AND tile_column = {} AND tile_row = {})",
                self.z, self.x, self.y
            ),
        }
    }

    #[must_use]
    pub fn json_arr_min(&self) -> String {
        format!("[{},{},{}]", self.x, self.y, self.z)
    }

    #[must_use]
    pub fn json_arr(&self) -> String {
        format!("[{}, {}, {}]", self.x, self.y, self.z)
    }

    #[must_use]
    pub fn json_obj(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    #[must_use]
    pub fn tuple_string(&self) -> String {
        format!("({}, {}, {})", self.x, self.y, self.z)
    }

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
                // let (east_merc, north_merc) = utiles::xy(east, north, Some(false));
                let (west_merc, south_merc) = xy(west, south, None);
                let (east_merc, north_merc) = xy(east, north, None);
                (west_merc, south_merc, east_merc, north_merc)
            }
            _ => (west, south, east, north),
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
        let id = match opts.fid.clone() {
            Some(fid) => fid,
            None => xyz,
        };
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

impl From<(u32, u32, u8)> for Tile {
    fn from(tuple: (u32, u32, u8)) -> Self {
        XYZ::from(tuple).into()
    }
}

impl From<&Map<String, Value>> for Tile {
    fn from(map: &Map<String, Value>) -> Self {
        let x = map["x"].as_u64().unwrap() as u32;
        let y = map["y"].as_u64().unwrap() as u32;
        let z = map["z"].as_u64().unwrap() as u8;
        utile!(x, y, z)
    }
}

impl From<&Vec<Value>> for Tile {
    fn from(arr: &Vec<Value>) -> Self {
        if arr.len() < 3 {
            panic!(
                "Invalid json value: {}",
                serde_json::to_string(&arr).unwrap()
            );
        }
        let x = arr[0].as_u64().unwrap() as u32;
        let y = arr[1].as_u64().unwrap() as u32;
        let z = arr[2].as_u64().unwrap() as u8;
        Tile::from((x, y, z))
    }
}
impl From<Vec<Value>> for Tile {
    fn from(arr: Vec<Value>) -> Self {
        Tile::from(&arr)
        //
        // if arr.len() < 3 {
        //     panic!(
        //         "Invalid json value: {}",
        //         serde_json::to_string(&arr).unwrap()
        //     );
        // }
        // let x = arr[0].as_u64().unwrap() as u32;
        // let y = arr[1].as_u64().unwrap() as u32;
        // let z = arr[2].as_u64().unwrap() as u8;
        // Tile::from((x, y, z))
    }
}

impl From<&Value> for Tile {
    fn from(val: &Value) -> Self {
        // is array? [x, y, z]
        match val {
            Value::Array(v) => {
                if v.len() < 3 {
                    panic!(
                        "Invalid json value: {}",
                        serde_json::to_string(&v).unwrap()
                    );
                }
                Tile::from(v)
                // let tuple = serde_json::from_value::<XYZ>(val).unwrap();
                // return Tile::from(tuple);
            }
            Value::Object(v) => {
                // if it has a "tile" key, use that
                // if has 'tile' key, use that
                if v.contains_key("tile")
                    && v["tile"].is_array()
                    && v["tile"].as_array().unwrap().len() == 3
                {
                    let tuple =
                        serde_json::from_value::<XYZ>(v["tile"].clone()).unwrap();
                    return Tile::from(tuple);
                }
                Tile::from(v)
            }
            _ => {
                panic!("Invalid json value: {val}");
            }
        }
    }
}

impl From<Value> for Tile {
    fn from(val: Value) -> Self {
        Tile::from(&val)
    }
}

impl From<&str> for Tile {
    fn from(s: &str) -> Self {
        Tile::from_json(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_obj() {
        let json_obj = r#"{"x": 1, "y": 2, "z": 3}"#;
        let tile = Tile::from_json_obj(json_obj);
        assert_eq!(tile, Tile::new(1, 2, 3));
    }

    #[test]
    fn parse_json_arr() {
        let json_arr = r#"[1, 2, 3]"#;
        let tile = Tile::from_json_arr(json_arr);
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
        let tile_from_obj = Tile::from(val_obj);
        assert_eq!(tile_from_obj, Tile::new(1, 2, 3));
    }

    #[test]
    fn tile_from_value_arr() {
        let json_arr = r#"[1, 2, 3]"#;
        let val_arr = serde_json::from_str::<Value>(json_arr).unwrap();
        let tile_from_arr = Tile::from(val_arr);
        assert_eq!(tile_from_arr, Tile::new(1, 2, 3));
    }
    #[test]
    fn tile_from_value_obj_with_array() {
        let json_obj_with_tile_array = r#"{"tile": [1, 2, 3]}"#;
        let val_obj_with_tile_array =
            serde_json::from_str::<Value>(json_obj_with_tile_array).unwrap();
        let tile_from_obj_with_tile_array = Tile::from(val_obj_with_tile_array);
        assert_eq!(tile_from_obj_with_tile_array, Tile::new(1, 2, 3));
    }
}
