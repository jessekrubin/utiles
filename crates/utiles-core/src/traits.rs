//! Utiles traits

use crate::TileLike;
use crate::parent::Parents;
use std::hash::Hash;

/// `IsOk` trait for checking if a value is Ok and returns a result
/// of self or an error
pub trait IsOk: Sized {
    /// Returns `Ok` if the value is `Ok` or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not `Ok`
    fn ok(&self) -> Result<Self, crate::UtilesCoreError>;
}

/// `Coord2d` like trait for 2D coordinates has x/y getters
pub trait Coord2dLike {
    /// Returns the x value
    fn x(&self) -> f64;

    /// Returns the y value
    fn y(&self) -> f64;
}

/// `LngLat` like trait
pub trait LngLatLike: Coord2dLike {
    /// Returns the longitude value
    fn lng(&self) -> f64;

    /// Returns the longitude value
    fn lon(&self) -> f64 {
        self.lng()
    }

    /// Returns the longitude value
    fn longitude(&self) -> f64 {
        self.lat()
    }

    /// Returns the latitude value
    fn lat(&self) -> f64;

    /// Returns the latitude value
    fn latitude(&self) -> f64 {
        self.lat()
    }
}

pub trait TileParent: Eq + Hash + Copy + TileLike {
    fn parent(&self, zoom: Option<u8>) -> Option<Self>
    where
        Self: Sized;

    fn iter_parents(&self) -> Parents<Self> {
        Parents {
            current: self.parent(None),
        }
    }

    #[must_use]
    fn root() -> Self
    where
        Self: Sized;
}

pub trait TileChildren1: Eq + Hash + Copy + TileLike {
    /// Returns direct children in Z order:
    ///     1) top-left
    ///     2) top-right
    ///     3) bottom-left
    ///     4) bottom-right
    #[must_use]
    fn children1(&self) -> [Self; 4];
}
