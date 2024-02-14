// /// Point2d and Point3d
// ///
// /// Previously this crate (utiles) was using the geo-types crate for it's Coord type and `coord!`
// /// macro. Yuri Astrakhan (https://github.com/nyurik) suggested not using geo-types given how
// /// simple the utiles use case is.
//
// /// PointLike trait for points
// pub trait PointLike {
//     /// Return the x value
//     fn x(&self) -> f64;
//
//     /// Return the y value
//     fn y(&self) -> f64;
// }
//
// /// Point2d struct for 2d f64 points
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Point2d {
//     /// x value
//     pub x: f64,
//
//     /// y value
//     pub y: f64,
// }
//
// /// Point3d struct for 3d f64 points
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Point3d {
//     /// x value
//     pub x: f64,
//
//     /// y value
//     pub y: f64,
//
//     /// z value
//     pub z: f64,
// }
//
// impl Point2d {
//     /// Create a new Point2d
//     #[must_use]
//     pub fn new(x: f64, y: f64) -> Self {
//         Point2d { x, y }
//     }
//
//     /// Return the x value
//     #[must_use]
//     pub fn x(&self) -> f64 {
//         self.x
//     }
//
//     /// Return the y value
//     #[must_use]
//     pub fn y(&self) -> f64 {
//         self.y
//     }
// }
//
// impl Point3d {
//     /// Create a new Point3d
//     #[must_use]
//     pub fn new(x: f64, y: f64, z: f64) -> Self {
//         Point3d { x, y, z }
//     }
//
//     /// Return the x value
//     #[must_use]
//     pub fn x(&self) -> f64 {
//         self.x
//     }
//
//     /// Return the y value
//     #[must_use]
//     pub fn y(&self) -> f64 {
//         self.y
//     }
//
//     /// Return the z value
//     #[must_use]
//     pub fn z(&self) -> f64 {
//         self.z
//     }
// }
use std::fmt::Debug;
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// Point2d struct for 2d points
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2d<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> {
    /// x value
    pub x: T,

    /// y value
    pub y: T,
}

/// Point3d struct for 3d points
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3d<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> {
    /// x value
    pub x: T,

    /// y value
    pub y: T,

    /// z value
    pub z: T,
}

impl<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> Point2d<T> {
    /// Create a new Point2d
    pub fn new(x: T, y: T) -> Self {
        Point2d { x, y }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }
}

impl<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> Point3d<T> {
    /// Create a new Point3d
    pub fn new(x: T, y: T, z: T) -> Self {
        Point3d { x, y, z }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    pub fn z(&self) -> T {
        self.z
    }
}
