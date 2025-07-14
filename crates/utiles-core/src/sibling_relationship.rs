//! Child index as related to its parent/sibling-tiles
use crate::tile::Tile;

/// Sibling relationship for tiles
pub enum SiblingRelationship {
    /// `UpperLeft` sibling
    UpperLeft = 0,
    /// `UpperRight` sibling
    UpperRight = 1,
    /// `LowerLeft` sibling
    LowerLeft = 2,
    /// `LowerRight` sibling
    LowerRight = 3,
}

impl From<(u32, u32)> for SiblingRelationship {
    fn from(value: (u32, u32)) -> Self {
        let is_left = value.0 % 2 == 0;
        let is_top = value.1 % 2 == 0;
        match (is_left, is_top) {
            (true, true) => Self::UpperLeft,
            (true, false) => Self::LowerLeft,
            (false, true) => Self::UpperRight,
            (false, false) => Self::LowerRight,
        }
    }
}

impl From<Tile> for SiblingRelationship {
    fn from(value: Tile) -> Self {
        let is_left = value.x % 2 == 0;
        let is_top = value.y % 2 == 0;
        match (is_left, is_top) {
            (true, true) => Self::UpperLeft,
            (true, false) => Self::LowerLeft,
            (false, true) => Self::UpperRight,
            (false, false) => Self::LowerRight,
        }
    }
}
