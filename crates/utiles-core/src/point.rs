use std::fmt::Debug;
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

/// Point2d struct for 2d points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point2d<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> {
    /// x value
    pub x: T,

    /// y value
    pub y: T,
}

/// Point3d struct for 3d points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point3d<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> {
    /// x value
    pub x: T,

    /// y value
    pub y: T,

    /// z value
    pub z: T,
}

impl<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> Point2d<T> {
    /// Create a new `Point2d`
    pub fn new(x: T, y: T) -> Self {
        Point2d { x, y }
    }

    /// Return the x value
    pub fn x(&self) -> T {
        self.x
    }

    /// Return the y value
    pub fn y(&self) -> T {
        self.y
    }
}

impl<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> Point3d<T> {
    /// Create a new `Point3d`
    pub fn new(x: T, y: T, z: T) -> Self {
        Point3d { x, y, z }
    }

    /// Return the x value
    pub fn x(&self) -> T {
        self.x
    }

    /// Return the y value
    pub fn y(&self) -> T {
        self.y
    }

    /// Return the z value
    pub fn z(&self) -> T {
        self.z
    }
}
