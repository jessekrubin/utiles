use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct XYZ(pub u32, pub u32, pub u8);

impl From<(u32, u32, u8)> for XYZ {
    fn from(xyz: (u32, u32, u8)) -> Self {
        XYZ(xyz.0, xyz.1, xyz.2)
    }
}
