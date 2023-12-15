/// Point2d and Point3d
///
/// Previously this crate (utiles) was using the geo-types crate for it's Coord type and `coord!`
/// macro. Yuri Astrakhan (https://github.com/nyurik) suggested not using geo-types given how
/// simple the utiles use case is.

pub trait PointLike {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}

/// Point2d struct for 2d f64 points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

/// Point3d struct for 3d f64 points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point2d {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Point2d { x, y }
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }
}

impl Point3d {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3d { x, y, z }
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub fn z(&self) -> f64 {
        self.z
    }
}
