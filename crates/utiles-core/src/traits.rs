//! Utiles traits

/// `IsOk` trait for checking if a value is Ok and returns a result
/// of self or an error
pub trait IsOk: Sized {
    /// Returns `Ok` if the value is `Ok` or an error
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
