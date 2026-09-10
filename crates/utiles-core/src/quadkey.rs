//! Quadkey conversion util(e)ities.
use std::iter::FusedIterator;

use crate::Tile;
use crate::errors::{UtilesCoreError, UtilesCoreResult};

/// Return the quadkey for a tile as a vector of u8 values (0, 1, 2, 3).
#[must_use]
pub fn xyz2quadkey_vec(x: u32, y: u32, z: u8) -> Vec<u8> {
    let mut qk_arr = Vec::with_capacity(z as usize);
    // let mut quadkey = String::new();
    for i in (0..z).rev() {
        let mut digit: u8 = 0;
        let mask = 1 << i;
        if (x & mask) != 0 {
            digit += 1;
        }
        if (y & mask) != 0 {
            digit += 2;
        }
        qk_arr.push(digit);
    }
    qk_arr
}

/// Return the quadkey for a tile as a string.
/// # Examples
/// ```
/// use utiles_core::xyz2quadkey;
/// let quadkey = xyz2quadkey(486, 332, 10);
/// assert_eq!(quadkey, "0313102310");
/// ```
#[must_use]
pub fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    QuadkeyCharsIter::new(Tile::new(x, y, z))
        .map(QuadkeyChar::as_char)
        .collect()
}
/// Return (x, y, z) for a quadkey as a tuple.
///
/// # Errors
///
/// Returns an error if the quadkey is invalid and cannot be converted to
/// tile xyz.
///
/// # Examples
/// ```
/// use utiles_core::quadkey2xyz;
/// let one_two_three = quadkey2xyz("123").unwrap();
/// assert_eq!(one_two_three, (5, 3, 3));
/// let xyz = quadkey2xyz("0313102310").unwrap();
/// assert_eq!(xyz, (486, 332, 10));
/// ```
pub fn quadkey2xyz(quadkey: &str) -> UtilesCoreResult<(u32, u32, u8)> {
    // Version with fold:
    //  quadkey.chars().try_fold((0, 0, 0), |(mut x, mut y, z), c| {
    //       x <<= 1;
    //       y <<= 1;
    //
    //       match c {
    //           '0' => Ok((x, y, z + 1)),
    //           '1' => Ok((x | 1, y, z + 1)),
    //           '2' => Ok((x, y | 1, z + 1)),
    //           '3' => Ok((x | 1, y | 1, z + 1)),
    //           _ => Err(UtilesError::InvalidQuadkey(c.to_string())),
    //       }
    //   })
    let mut x = 0;
    let mut y = 0;
    let mut z = 0;
    for c in quadkey.chars() {
        x <<= 1;
        y <<= 1;
        z += 1;
        match c {
            '0' => {}
            '1' => {
                x += 1;
            }
            '2' => {
                y += 1;
            }
            '3' => {
                x += 1;
                y += 1;
            }
            _ => {
                return Err(UtilesCoreError::InvalidQuadkey(c.to_string()));
            }
        }
    }
    Ok((x, y, z))
}

/// Return Tile struct from quadkey string
///
/// # Errors
///
/// Returns an error if the quadkey is invalid and cannot be converted to
/// tile xyz.
///
/// # Examples
/// ```
/// use utiles_core::{Tile, quadkey2tile};
/// let tile = quadkey2tile("0313102310").unwrap();
/// assert_eq!(tile, Tile::new(486, 332, 10));
/// ```
pub fn quadkey2tile(quadkey: &str) -> UtilesCoreResult<Tile> {
    let xyz = quadkey2xyz(quadkey)?;
    Ok(Tile::new(xyz.0, xyz.1, xyz.2))
}

/// Return y-flipped quadkey
#[must_use]
pub fn quadkey_flipy(quadkey: &str) -> String {
    let mut quadkey_flipped = String::new();
    for c in quadkey.chars().map(QuadkeyChar::from_char) {
        if let Some(c) = c {
            quadkey_flipped.push(c.flipy().as_char());
        } else {
            return String::new();
        }
    }
    quadkey_flipped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum QuadkeyChar {
    Qk0 = 0,
    Qk1 = 1,
    Qk2 = 2,
    Qk3 = 3,
}

impl QuadkeyChar {
    #[inline]
    const fn as_char(self) -> char {
        match self {
            Self::Qk0 => '0',
            Self::Qk1 => '1',
            Self::Qk2 => '2',
            Self::Qk3 => '3',
        }
    }

    const fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(Self::Qk0),
            '1' => Some(Self::Qk1),
            '2' => Some(Self::Qk2),
            '3' => Some(Self::Qk3),
            _ => None,
        }
    }

    const fn flipy(self) -> Self {
        match self {
            Self::Qk0 => Self::Qk2,
            Self::Qk1 => Self::Qk3,
            Self::Qk2 => Self::Qk0,
            Self::Qk3 => Self::Qk1,
        }
    }
}

struct QuadkeyCharsIter {
    tile: Tile,
    index: u8,
}

impl QuadkeyCharsIter {
    const fn new(tile: Tile) -> Self {
        Self { tile, index: 0 }
    }
}

impl Iterator for QuadkeyCharsIter {
    type Item = QuadkeyChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tile.z {
            return None;
        }
        let mask = 1 << (self.tile.z - 1 - self.index);
        let right = (self.tile.x & mask) != 0;
        let down = (self.tile.y & mask) != 0;
        self.index += 1;
        match (right, down) {
            (false, false) => Some(QuadkeyChar::Qk0),
            (true, false) => Some(QuadkeyChar::Qk1),
            (false, true) => Some(QuadkeyChar::Qk2),
            (true, true) => Some(QuadkeyChar::Qk3),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.tile.z - self.index;
        (remaining as usize, Some(remaining as usize))
    }
}

impl ExactSizeIterator for QuadkeyCharsIter {
    fn len(&self) -> usize {
        (self.tile.z - self.index) as usize
    }
}

impl FusedIterator for QuadkeyCharsIter {}
