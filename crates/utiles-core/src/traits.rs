/// BoundingBox like trait
pub trait BoundingBoxLike {
    /// Returns west/left bound
    fn west(&self) -> f64;

    /// Returns south/bottom bound
    fn south(&self) -> f64;

    /// Returns east/right bound
    fn east(&self) -> f64;

    /// Returns north/top bound
    fn north(&self) -> f64;

    /// Returns the width of the bounding box
    fn left(&self) -> f64;

    /// Returns the height of the bounding box
    fn bottom(&self) -> f64;

    /// Returns the width of the bounding box
    fn right(&self) -> f64;

    /// Returns the height of the bounding box
    fn top(&self) -> f64;
}
