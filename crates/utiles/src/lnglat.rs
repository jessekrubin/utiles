use geo_types::{coord, Coord};

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
    #[must_use]
    pub fn new(lng: f64, lat: f64) -> Self {
        LngLat {
            xy: coord! { x: lng, y: lat},
        }
    }

    #[must_use]
    pub fn lng(&self) -> f64 {
        self.xy.x
    }

    #[must_use]
    pub fn lat(&self) -> f64 {
        self.xy.y
    }

    #[must_use]
    pub fn lon(&self) -> f64 {
        self.xy.x
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.xy.x
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.xy.y
    }
}

impl std::fmt::Display for LngLat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.xy.x, self.xy.y)
    }
}

impl Iterator for LngLat {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let lng = self.xy.x;
        let lat = self.xy.y;
        self.xy.x += 1.0;
        self.xy.y += 1.0;
        Some((lng, lat))
    }
}
