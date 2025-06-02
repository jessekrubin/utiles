use crate::tile_like::TileLike;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct TileTuple(pub u32, pub u32, pub u8);

impl TileTuple {
    #[must_use]
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Self(x, y, z)
    }
}

impl From<(u32, u32, u8)> for TileTuple {
    fn from(xyz: (u32, u32, u8)) -> Self {
        TileTuple(xyz.0, xyz.1, xyz.2)
    }
}
impl From<(u8, u32, u32)> for TileTuple {
    fn from(xyz: (u8, u32, u32)) -> Self {
        TileTuple(xyz.1, xyz.2, xyz.0)
    }
}

impl TileLike for TileTuple {
    fn x(&self) -> u32 {
        self.0
    }

    fn y(&self) -> u32 {
        self.1
    }

    fn z(&self) -> u8 {
        self.2
    }
}
