use std::fmt::Display;
use std::str::FromStr;

use crate::point::Point2d;
use crate::traits::{Coord2dLike, IsOk, LngLatLike};
use crate::UtilesCoreResult;

/// `LngLat` contains a longitude and latitude as f64.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LngLat {
    /// 2d point - x -> longitude, y -> latitude
    pub xy: Point2d<f64>,
}

impl LngLat {
    /// Create a new `LngLat` from longitude & latitude.
    #[must_use]
    pub fn new(lng: f64, lat: f64) -> Self {
        LngLat {
            xy: Point2d::new(lng, lat),
        }
    }

    /// Return the x/lng/lon/long/longitude value.
    #[must_use]
    pub fn lng(&self) -> f64 {
        self.xy.x
    }

    /// Return the y/lat/latitude value.
    #[must_use]
    pub fn lat(&self) -> f64 {
        self.xy.y
    }

    /// Return the x/lng/lon/long/longitude value.
    #[must_use]
    pub fn lon(&self) -> f64 {
        self.xy.x
    }

    /// Return the y/lat/latitude value.
    #[must_use]
    pub fn x(&self) -> f64 {
        self.xy.x
    }

    /// Return the y/lat/latitude value.
    #[must_use]
    pub fn y(&self) -> f64 {
        self.xy.y
    }
}

impl Coord2dLike for LngLat {
    fn x(&self) -> f64 {
        self.xy.x
    }

    fn y(&self) -> f64 {
        self.xy.y
    }
}

impl LngLatLike for LngLat {
    fn lng(&self) -> f64 {
        self.xy.x
    }

    fn lat(&self) -> f64 {
        self.xy.y
    }
}

impl IsOk for LngLat {
    fn ok(&self) -> UtilesCoreResult<Self> {
        if self.xy.x >= -180.0
            && self.xy.x <= 180.0
            && self.xy.y >= -90.0
            && self.xy.y <= 90.0
        {
            Ok(*self)
        } else {
            Err(crate::UtilesCoreError::InvalidLngLat(format!(
                "Invalid LngLat coordinates: x: {}, y: {}",
                self.xy.x, self.xy.y
            )))
        }
    }
}

impl Display for LngLat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.xy.x, self.xy.y)
    }
}

impl IntoIterator for LngLat {
    type Item = (f64, f64);
    type IntoIter = std::iter::Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once((self.xy.x, self.xy.y))
    }
}

impl From<(f64, f64)> for LngLat {
    fn from(xy: (f64, f64)) -> Self {
        LngLat::new(xy.0, xy.1)
    }
}

impl FromStr for LngLat {
    type Err = std::num::ParseFloatError; // Change this to your correct Error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split the string, parse the parts into float and return LngLat.
        let parts: Vec<&str> = s.split(',').collect();
        // parse parts to float
        let x = parts[0].parse::<f64>()?;
        let y = parts[1].parse::<f64>()?;
        Ok(LngLat::new(x, y))
    }
}
