use crate::errors::UtilesCoreResult;
use crate::UtilesCoreError;
use std::num::ParseIntError;
use std::ops::BitAnd;

use crate::UtilesCoreError::InvalidZoom;

type Zooms = Vec<u8>;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ZoomSet(u32);

#[must_use]
pub fn zset2zvec(zset: u32) -> Vec<u8> {
    (0..32)
        .rev()
        .filter(|&i| (zset & (1 << i)) != 0)
        .map(|i| (31 - i) as u8) // Convert bit index to zoom level and cast to u8
        .collect()
}

#[must_use]
pub fn zvec2zset(zvec: Vec<u8>) -> u32 {
    zvec.iter().fold(0, |acc, &z| acc | (1 << (31 - z)))
}

/// ZoomSet is a set of zoom levels represented as a u32
///
/// # Examples
/// ```
/// use utiles_core::zoom::ZoomSet;
/// let zset = ZoomSet::new(0b11100000_00000000_00000000_00000000);
/// let zooms_vec: Vec<u8> = zset.into();
/// assert_eq!(zooms_vec, vec![0, 1, 2]);
/// ```
impl ZoomSet {
    #[must_use]
    pub fn new(zset: u32) -> Self {
        Self(zset)
    }

    #[must_use]
    pub fn from_zooms(zooms: Vec<u8>) -> Self {
        Self(zvec2zset(zooms))
    }

    #[must_use]
    pub fn to_zooms(&self) -> Vec<u8> {
        zset2zvec(self.0)
    }
}

impl From<u8> for ZoomSet {
    fn from(zoom: u8) -> Self {
        ZoomSet(1 << (31 - zoom))
    }
}

impl From<u32> for ZoomSet {
    fn from(zset: u32) -> Self {
        ZoomSet(zset)
    }
}

impl TryFrom<Vec<u8>> for ZoomSet {
    type Error = UtilesCoreError;

    fn try_from(zvec: Vec<u8>) -> Result<Self, Self::Error> {
        let result = zvec.iter().try_fold(0u32, |acc, &z| {
            if z > 31 {
                Err(InvalidZoom(z.to_string()))
            } else {
                Ok(acc | (1 << (31 - z)))
            }
        })?;
        Ok(ZoomSet::new(result)) // Replace with actual construction method
    }
}

impl BitAnd for ZoomSet {
    type Output = ZoomSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        ZoomSet(self.0 & rhs.0)
    }
}

impl BitAnd<u32> for ZoomSet {
    type Output = ZoomSet;

    fn bitand(self, rhs: u32) -> Self::Output {
        ZoomSet(self.0 & rhs)
    }
}

impl BitAnd<ZoomSet> for u32 {
    type Output = ZoomSet;

    fn bitand(self, rhs: ZoomSet) -> Self::Output {
        ZoomSet(self & rhs.0)
    }
}

impl BitAnd<u8> for ZoomSet {
    type Output = ZoomSet;

    fn bitand(self, rhs: u8) -> Self::Output {
        ZoomSet(self.0 & (1 << (31 - rhs)))
    }
}

impl From<ZoomSet> for Vec<u8> {
    fn from(zset: ZoomSet) -> Self {
        zset2zvec(zset.0)
    }
}

type ZoomsSetInt = u32;

pub struct ZoomRange {
    pub minz: u8,
    pub maxz: u8,
}

// default zoom range
impl Default for ZoomRange {
    fn default() -> Self {
        Self { minz: 0, maxz: 31 }
    }
}

impl ZoomRange {
    #[must_use]
    pub fn new(min: u8, max: u8) -> Self {
        Self {
            minz: min,
            maxz: max,
        }
    }

    #[must_use]
    pub fn from_max(max: u8) -> Self {
        Self { minz: 0, maxz: max }
    }

    #[must_use]
    pub fn from_min(min: u8) -> Self {
        Self {
            minz: min,
            maxz: 31,
        }
    }
}

/// convert range of zoom levels to a set of zoom levels
///
/// # Examples
/// ```
/// use utiles_core::zoom::{ZoomRange, ZoomSet};
/// let zrange = ZoomRange::new(0, 7);
/// let zset: ZoomSet = zrange.into();
/// assert_eq!(zset, ZoomSet::new(0b11111111_00000000_00000000_00000000));
/// ```
impl From<ZoomRange> for ZoomSet {
    fn from(zoom_range: ZoomRange) -> Self {
        ZoomSet(
            (zoom_range.minz..=zoom_range.maxz)
                .rev()
                .fold(0, |acc, z| acc | (1 << (31 - z))),
        )
    }
}

/// parse a string of zoom levels to a vector of zoom levels
/// # Examples
/// ```
/// use utiles_core::zoom::parse_zooms;
/// let zvec = parse_zooms("0,1,2,3,4,5,6,7");
/// assert_eq!(zvec.unwrap(), vec![0, 1, 2, 3, 4, 5, 6, 7]);
/// ```
///
/// ```
/// use utiles_core::zoom::parse_zooms;
/// let zvec = parse_zooms("0-7");
/// assert_eq!(zvec.unwrap(), vec![0, 1, 2, 3, 4, 5, 6, 7]);
/// ```
///
/// ```
/// use utiles_core::zoom::parse_zooms;
/// let zvec = parse_zooms("0-2,4-7");
/// assert_eq!(zvec.unwrap(), vec![0, 1, 2, 4, 5, 6, 7]);
/// ```
///
/// ```
/// use utiles_core::zoom::parse_zooms;
/// let zvec = parse_zooms("1,2,3,4,4,4,4,4");
/// assert_eq!(zvec.unwrap(), vec![1, 2, 3, 4]);
/// ```
pub fn parse_zooms(zstr: &str) -> UtilesCoreResult<Vec<u8>> {
    let mut zvec: Vec<u8> = vec![];
    for z in zstr.split(',') {
        if z.contains('-') {
            let zrange: Result<Vec<u8>, ParseIntError> = z
                .split('-')
                .map(|z| z.parse::<u8>())
                .collect::<Result<Vec<_>, _>>();

            let zrange = match zrange {
                Ok(zrange) => match zrange.len() {
                    1 => vec![zrange[0]],
                    2 => (zrange[0]..=zrange[1]).collect(),
                    _ => vec![],
                },
                Err(_) => return Err(InvalidZoom(z.to_string())),
            };
            zvec.extend(zrange);
        } else {
            match z.parse::<u8>() {
                Ok(num) => zvec.push(num),
                Err(_) => return Err(InvalidZoom(z.to_string())),
            }
        }
    }
    // make sure zooms are between 0 and 32
    for z in &zvec {
        if *z > 32 {
            return Err(InvalidZoom((*z).to_string()));
        }
    }
    // unique and sort zooms
    zvec.sort();
    zvec.dedup();
    Ok(zvec)
}

pub enum ZoomOrZooms {
    Zoom(u8),
    Zooms(Vec<u8>),
}

pub struct ZoomsSet(ZoomsSetInt);

impl From<u8> for ZoomOrZooms {
    fn from(zoom: u8) -> Self {
        ZoomOrZooms::Zoom(zoom)
    }
}

impl From<Vec<u8>> for ZoomOrZooms {
    fn from(zooms: Vec<u8>) -> Self {
        ZoomOrZooms::Zooms(zooms)
    }
}

impl From<ZoomOrZooms> for Zooms {
    fn from(zoom_or_zooms: ZoomOrZooms) -> Self {
        match zoom_or_zooms {
            ZoomOrZooms::Zoom(zoom) => vec![zoom],
            ZoomOrZooms::Zooms(zooms) => zooms,
        }
    }
}

impl From<ZoomOrZooms> for ZoomsSetInt {
    fn from(zoom_or_zooms: ZoomOrZooms) -> Self {
        match zoom_or_zooms {
            ZoomOrZooms::Zoom(zoom) => 1 << (31 - zoom),
            ZoomOrZooms::Zooms(zooms) => zvec2zset(zooms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zset2zvec_none() {
        let zset: u32 = 0b00000000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![];
        assert_eq!(zset2zvec(zset), zvec);
    }

    #[test]
    fn zset2zvec_0_1_2() {
        let zset: u32 = 0b11100000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![0, 1, 2];

        assert_eq!(zset2zvec(zset), zvec);
    }

    #[test]
    fn zset2zvec_all() {
        let zset: u32 = 0b11111111_11111111_11111111_11111111; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];
        assert_eq!(zset2zvec(zset), zvec);
    }

    #[test]
    fn zvec2zset_none() {
        let zset: u32 = 0b00000000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![];
        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_0_1_2() {
        let zset: u32 = 0b11100000_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![0, 1, 2];

        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_all() {
        let zset: u32 = 0b11111111_11111111_11111111_11111111; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
        ];
        assert_eq!(zvec2zset(zvec), zset);
    }

    #[test]
    fn zvec2zset_0_1_2_3_4_5_6_7() {
        let zset: u32 = 0b11111111_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zvec: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let zset_from_zvec = zvec2zset(zvec);
        assert_eq!(zset_from_zvec, zset);
    }

    #[test]
    fn zoom_set_into_zoom_vec() {
        let zset_int: u32 = 0b11111111_00000000_00000000_00000000; // Example, zoom levels 2 and 4 are set
        let zset: ZoomSet = zset_int.into();
        let zvec = Vec::from(zset);
        assert_eq!(zvec, vec![0, 1, 2, 3, 4, 5, 6, 7]);

        // let zvec: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        // let zset_from_zvec = zvec2zset(zvec);
        // assert_eq!(zset_from_zvec, zset);
    }
}
