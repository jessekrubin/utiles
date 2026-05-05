//! 2d/3d points
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
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Return the x value
    pub const fn x(&self) -> T {
        self.x
    }

    /// Return the y value
    pub const fn y(&self) -> T {
        self.y
    }
}

impl<T: Copy + PartialOrd + PartialEq + Debug + Add + Sub> Point3d<T> {
    /// Create a new `Point3d`
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// Return the x value
    pub const fn x(&self) -> T {
        self.x
    }

    /// Return the y value
    pub const fn y(&self) -> T {
        self.y
    }

    /// Return the z value
    pub const fn z(&self) -> T {
        self.z
    }
}
