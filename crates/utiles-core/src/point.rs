/// Point2d and Point3d
///
/// Previously this crate (utiles) was using the geo-types crate for it's Coord type and `coord!`
/// macro. Yuri Astrakhan (https://github.com/nyurik) suggested not using geo-types given how
/// simple the utiles use case is.

/// PointLike trait for points
pub trait PointLike {
    /// Return the x value
    fn x(&self) -> f64;

    /// Return the y value
    fn y(&self) -> f64;
}

/// Point2d struct for 2d f64 points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2d {
    /// x value
    pub x: f64,

    /// y value
    pub y: f64,
}

/// Point3d struct for 3d f64 points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3d {
    /// x value
    pub x: f64,

    /// y value
    pub y: f64,

    /// z value
    pub z: f64,
}

impl Point2d {
    /// Create a new Point2d
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Point2d { x, y }
    }

    /// Return the x value
    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Return the y value
    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }
}

impl Point3d {
    /// Create a new Point3d
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3d { x, y, z }
    }

    /// Return the x value
    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Return the y value
    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Return the z value
    #[must_use]
    pub fn z(&self) -> f64 {
        self.z
    }
}
