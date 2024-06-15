//! Zoom levels, zoom-collections and ranges oh my!
use crate::constants::MAX_ZOOM;
use serde::Serialize;
use std::num::ParseIntError;
use std::ops::BitAnd;

use crate::errors::UtilesCoreResult;
use crate::UtilesCoreError;
use crate::UtilesCoreError::InvalidZoom;

/// `ZoomSet` is a set of zoom levels represented as a 32-bit unsigned integer
/// where each bit represents a zoom level (0 <= z <= 30). BY DEFAULT: The
/// least significant bit represents zoom level 0 and the SECOND most significant
/// bit represents zoom level 30. BUT if the MOST significant bit is 1, then the
/// order is reversed.
///
/// # Examples
/// ```
/// use utiles_core::zoom::ZoomSet;
/// let zset_int_fwd = ZoomSet::new(0b0000_0000_0000_0000_0000_0000_0000_0111);
/// //                            ^ is 1 so the order is forward/default
/// let zset_int_rev = ZoomSet::new(0b1111_0000_0000_0000_0000_0000_0000_0000);
/// //                            ^ is 1 so the order is reversed
/// let zooms_fwd_vec: Vec<u8> = zset_int_fwd.into();
/// assert_eq!(zooms_fwd_vec, vec![0, 1, 2]);
/// let zooms_rev_vec: Vec<u8> = zset_int_rev.into();
/// assert_eq!(zooms_rev_vec, vec![0, 1, 2]);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize)]
pub struct ZoomSet(u32);

/// Return a vector of zoom levels from a zoom-set u32
///
/// # Examples
///
/// ```
/// use utiles_core::zoom::zset2zvec;
/// let zset: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0111;
/// let zvec: Vec<u8> = zset2zvec(zset);
/// assert_eq!(zvec, vec![0, 1, 2]);
/// ```
///
/// ```
/// use utiles_core::zoom::zset2zvec;
/// let zset: u32 = 0b1111_0000_0000_0000_0000_0000_0000_0000;
/// let zvec: Vec<u8> = zset2zvec(zset);
/// assert_eq!(zvec, vec![0, 1, 2]);
/// ```
#[must_use]
pub fn zset2zvec(zset: u32) -> Vec<u8> {
    // if the most significant bit is 1, then the order is reversed
    if zset & (1 << 31) != 0 {
        zset2zvec(zset.reverse_bits() >> 1)
    } else {
        (0u8..31).filter(|&i| (zset & (1u32 << i)) != 0).collect()
    }
}

/// Return a zoom-set u32 from a vector of zoom levels
///
/// # Examples
/// ```
/// use utiles_core::zoom::zvec2zset;
/// let zvec: Vec<u8> = vec![0, 1, 2];
/// let zset_int: u32 = zvec2zset(&zvec);
/// assert_eq!(zset_int, 0b0000_0000_0000_0000_0000_0000_0000_0111);
/// ```
#[must_use]
pub fn zvec2zset(zvec: &[u8]) -> u32 {
    zvec.iter().fold(0, |acc, &z| acc | (1 << z))
}

/// Return a vector of zoom levels from a reversed-zoom-set u32
///
#[must_use]
pub fn zset2zvec_rev(zset: u32) -> Vec<u8> {
    zset2zvec(zset.reverse_bits() >> 1)
}

/// Return a zoom-set u32 from a vector of zoom levels
#[must_use]
pub fn zvec2zset_rev(zvec: &[u8]) -> u32 {
    zvec.iter().fold(0, |acc, &z| acc | (1 << (31 - z)))
}

/// `ZoomSet` implementation
impl ZoomSet {
    /// Create a new `ZoomSet` from a u32
    #[must_use]
    pub fn new(zset: u32) -> Self {
        Self(zset)
    }

    /// Create a new `ZoomSet` from a vector of zoom levels
    #[must_use]
    pub fn from_zooms(zooms: &[u8]) -> Self {
        Self(zvec2zset(zooms))
    }

    /// Return a vector of zoom levels from a zoom-set u32
    #[must_use]
    pub fn to_zooms(&self) -> Vec<u8> {
        zset2zvec(self.0)
    }

    #[must_use]
    pub fn all() -> Self {
        Self(0b0111_1111_1111_1111_1111_1111_1111_1111)
    }

    #[must_use]
    pub fn zoom_ranges(&self) -> Vec<ZoomRange> {
        let mut ranges: Vec<ZoomRange> = vec![];
        let mut min: u8 = 0;
        let mut max: u8 = 0;
        let mut i: u8 = 0;
        while i < MAX_ZOOM {
            if self.0 & (1 << i) != 0 {
                if min == 0 {
                    min = i;
                }
                max = i;
            } else if min != 0 {
                ranges.push(ZoomRange::new(min, max));
                min = 0;
                max = 0;
            }
            i += 1;
        }
        if min != 0 {
            ranges.push(ZoomRange::new(min, max));
        }
        ranges
    }
}

impl From<u8> for ZoomSet {
    fn from(zoom: u8) -> Self {
        ZoomSet(1 << zoom)
    }
}

impl From<u32> for ZoomSet {
    fn from(zset: u32) -> Self {
        ZoomSet(zset)
    }
}

impl From<ZoomSet> for Vec<u8> {
    fn from(zset: ZoomSet) -> Self {
        zset2zvec(zset.0)
    }
}

impl TryFrom<Vec<u8>> for ZoomSet {
    type Error = UtilesCoreError;

    fn try_from(zvec: Vec<u8>) -> Result<Self, Self::Error> {
        let result = zvec.iter().try_fold(0u32, |acc, &z| {
            if z > MAX_ZOOM {
                Err(InvalidZoom(z.to_string()))
            } else {
                Ok(acc | (1 << z))
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

type ZoomsSetInt = u32;

/// Struct representing zoom level range (min, max)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ZoomRange {
    /// Minimum zoom level
    pub min: u8,
    /// Maximum zoom level
    pub max: u8,
}

// default zoom range
impl Default for ZoomRange {
    fn default() -> Self {
        Self {
            min: 0,
            max: MAX_ZOOM,
        }
    }
}

impl ZoomRange {
    /// Create a new `ZoomRange`
    #[must_use]
    pub fn new(min: u8, max: u8) -> Self {
        Self { min, max }
    }

    /// Create a new `ZoomRange` from a maximum zoom level (0 to max)
    #[must_use]
    pub fn from_max(max: u8) -> Self {
        Self { min: 0, max }
    }

    /// Create a new `ZoomRange` from a minimum zoom level (min to 30)
    #[must_use]
    pub fn from_min(min: u8) -> Self {
        Self { min, max: 30 }
    }
}

impl IntoIterator for ZoomRange {
    type Item = u8;
    type IntoIter = std::ops::RangeInclusive<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.min..=self.max
    }
}

/// Convert range of zoom levels to a set of zoom levels
///
/// # Examples
/// ```
/// use utiles_core::zoom::{ZoomRange, ZoomSet};
/// let zrange = ZoomRange::new(0, 7);
/// let zset: ZoomSet = zrange.into();
/// assert_eq!(zset, ZoomSet::new(0b0000_0000_0000_0000_0000_0000_1111_1111));
/// ```
impl From<ZoomRange> for ZoomSet {
    fn from(zoom_range: ZoomRange) -> Self {
        ZoomSet(zoom_range.into_iter().fold(0, |acc, z| acc | (1 << z)))
    }
}

pub fn parse_zooms(zstr: &str) -> UtilesCoreResult<Vec<u8>> {
    let mut zvec: Vec<u8> = vec![];
    for z in zstr.split(',') {
        if z.contains('-') {
            let zrange: Result<Vec<u8>, ParseIntError> =
                z.split('-').map(str::parse).collect::<Result<Vec<_>, _>>();

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
    zvec.sort_unstable();
    zvec.dedup();
    Ok(zvec)
}

/// Enum representing a single zoom level or a vec of zoom levels
pub enum ZoomOrZooms {
    /// A single zoom level
    Zoom(u8),

    /// A vec of zoom levels
    Zooms(Vec<u8>),
}

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

impl From<ZoomOrZooms> for ZoomsSetInt {
    fn from(zoom_or_zooms: ZoomOrZooms) -> Self {
        match zoom_or_zooms {
            ZoomOrZooms::Zoom(zoom) => 1 << (31 - zoom),
            ZoomOrZooms::Zooms(zooms) => zvec2zset_rev(&zooms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zset2zvec_none() {
        let zset: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
        let zvec: Vec<u8> = vec![];
        assert_eq!(zset2zvec(zset), zvec);
        assert_eq!(zset2zvec_rev(zset), zvec);
        assert_eq!(zset, 0);
    }

    #[test]
    fn zset2zvec_0_1_2() {
        let zset_fwd: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0111;
        let zset_rev: u32 = 0b1111_0000_0000_0000_0000_0000_0000_0000;
        let zvec: Vec<u8> = vec![0, 1, 2];
        assert_eq!(zset2zvec(zset_fwd), zvec);
        assert_eq!(zset_fwd, 7);
        assert_eq!(zset2zvec_rev(zset_rev), zvec);
    }

    #[test]
    fn zset2zvec_all() {
        let zset_int_fwd: u32 = 0b0111_1111_1111_1111_1111_1111_1111_1111;
        let zset_int_rev: u32 = 0b1111_1111_1111_1111_1111_1111_1111_1111;
        let zvec: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
        ];
        assert_eq!(zset2zvec(zset_int_fwd), zvec);
        assert_eq!(zset2zvec_rev(zset_int_rev), zvec);
    }

    #[test]
    fn zvec2zset_none() {
        let zset_int: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
        assert!(zset2zvec(zset_int).is_empty());
        assert!(zset2zvec_rev(zset_int).is_empty());
    }

    #[test]
    fn zvec2zset_0_1_2() {
        let zset_int: u32 = 0b1110_0000_0000_0000_0000_0000_0000_0000;
        let zvec: Vec<u8> = vec![0, 1, 2];

        assert_eq!(zvec2zset_rev(&zvec), zset_int);
    }

    #[test]
    fn zvec2zset_0_1_2_3_4_5_6_7() {
        let zset_int: u32 = 0b1111_1111_0000_0000_0000_0000_0000_0000;
        let zvec: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let zset_from_zvec = zvec2zset_rev(&zvec);
        assert_eq!(zset_from_zvec, zset_int);
    }

    #[test]
    fn zoom_set_into_zoom_vec() {
        let zset_int_fwd: u32 = 0b0000_0000_0000_0000_0000_0000_1111_1111;
        let zet_fwd_vec: Vec<u8> = ZoomSet::from(zset_int_fwd).into();
        assert_eq!(zet_fwd_vec, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let zset_int_rev: u32 = 0b1111_1111_1000_0000_0000_0000_0000_0000;
        let zset: ZoomSet = zset_int_rev.into();
        let zvec = Vec::from(zset);
        assert_eq!(zvec, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
