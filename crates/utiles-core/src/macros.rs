/// Tile macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to figure out how to write a macro? yes
#[macro_export]
macro_rules! utile {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, $y, $z)
    };
}

#[macro_export]
macro_rules! utile_yup {
    ($x:expr, $y:expr, $z:expr) => {
        Tile::new($x, flipy($y, $z), $z)
    };
}

/// point2d macro to create a new point.
/// Replacement for coord! macro from geo-types
///
/// # Examples
///
/// ```
/// use utiles_core::{point2d, Point2d};
/// let p = point2d!{ x: 1.0, y: 2.0 };
/// assert_eq!(p.x(), 1.0);
/// assert_eq!(p.y(), 2.0);
/// ```
#[macro_export]
macro_rules! point2d {
    { x: $x:expr, y: $y:expr } => {
        Point2d::new($x, $y)
    };
}
