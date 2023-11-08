use std::cmp::Ordering;
use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::bbox::BBox;
use crate::constants::EPSILON;
use crate::lnglat::LngLat;
use crate::{
    bounds, children, flipy, ll, lr, neighbors, parent, pmtiles, quadkey2tile,
    siblings, traits, ul, ur, xyz2quadkey, XYZ,
};

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
        if s.starts_with('{'){ // if '{' assume its an obj
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
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Tile { x, y, z }
    }

    #[allow(dead_code)]
    pub fn valid(&self) -> bool {
        crate::valid(self.x, self.y, self.z)
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
        pmtiles::xyz2pmid(self.x, self.y, self.z)
    }

    pub fn from_pmtileid(id: u64) -> Self {
        let (x, y, z) = pmtiles::pmid2xyz(id);
        Tile::new(x, y, z)
    }

    pub fn fmt_zxy(&self, sep: Option<&str>) -> String {
        match sep {
            Some(sep) => format!("{}{}{}{}{}", self.z, sep, self.x, sep, self.y),
            None => format!("{}/{}/{}", self.z, self.x, self.y),
        }
    }

    pub fn fmt_zxy_ext(&self, ext: &str, sep: Option<&str>) -> String {
        match sep {
            Some(sep) => {
                format!("{}{}{}{}{}.{}", self.z, sep, self.x, sep, self.y, ext)
            }
            None => format!("{}/{}/{}.{}", self.z, self.x, self.y, ext),
        }
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

    pub fn from_json_obj(json: &str) -> Self {
        let res = serde_json::from_str(json);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid json_arr: {e}");
            }
        }
    }

    pub fn from_json_arr(json: &str) -> Self {
        let res = serde_json::from_str(json);
        match res {
            Ok(tile) => tile,
            Err(e) => {
                panic!("Invalid json_arr: {e}");
            }
        }
    }

    pub fn from_json(json: &str) -> Self {
        if json.starts_with('[') {
            return Self::from_json_arr(json);
        }
        Self::from_json_obj(json)
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

    pub fn json_arr_min(&self) -> String {
        format!("[{},{},{}]", self.x, self.y, self.z)
    }

    pub fn json_arr(&self) -> String {
        format!("[{}, {}, {}]", self.x, self.y, self.z)
    }

    pub fn json_obj(&self) -> String {
        serde_json::to_string(self).unwrap()
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
        let tile =  quadkey.parse::<Tile>();
        assert_eq!(tile.unwrap(), Tile::new(81, 197, 9));
    }
}