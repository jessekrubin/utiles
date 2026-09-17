//! Quadkey conversion util(e)ities.
use std::iter::FusedIterator;

use crate::Tile;
use crate::errors::UtilesCoreResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadkeyError {
    InvalidChar,
    InvalidLength(usize),
}

impl std::fmt::Display for QuadkeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidChar => write!(f, "Invalid char in quadkey"),
            Self::InvalidLength(len) => {
                write!(f, "Invalid length for quadkey: {len}")
            }
        }
    }
}

impl std::error::Error for QuadkeyError {}

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
pub fn quadkey2xyz(quadkey: &str) -> Result<(u32, u32, u8), QuadkeyError> {
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
    QuadkeyRef::try_new(quadkey).map(|qk| qk.to_xyz())
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

const QK_CHAR_0: u8 = 0b0011_0000;
const QK_CHAR_1: u8 = 0b0011_0001;
const QK_CHAR_2: u8 = 0b0011_0010;
const QK_CHAR_3: u8 = 0b0011_0011;
const QK_CHAR_MASK: u8 = !((QK_CHAR_0 & QK_CHAR_1 & QK_CHAR_2 & QK_CHAR_3)
    ^ (QK_CHAR_0 | QK_CHAR_1 | QK_CHAR_2 | QK_CHAR_3));

/// Return `true` if the given byte is a valid byte ('0', '1', '2', or '3')
///
/// # Examples
///
/// ```rust
/// use utiles_core::is_quadkey_byte;
///
/// assert!(is_quadkey_byte(b'0'));
/// assert!(is_quadkey_byte(b'1'));
/// assert!(is_quadkey_byte(b'2'));
/// assert!(is_quadkey_byte(b'3'));
/// assert!(!is_quadkey_byte(b'4'));
/// ```
///
#[inline]
#[must_use]
pub const fn is_quadkey_byte(b: u8) -> bool {
    b & QK_CHAR_MASK == QK_CHAR_0
}

/// Return `true` if the given string is valid quadkey, `false` otherwise.
///
/// ```rust
/// use utiles_core::is_quadkey;
///
/// assert!(is_quadkey("0123"));
/// assert!(!is_quadkey("0456"));
/// ```
#[inline]
#[must_use]
pub fn is_quadkey(s: &str) -> bool {
    s.len() < 30 && s.bytes().all(|b| b & QK_CHAR_MASK == QK_CHAR_0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadkeyRef<'a>(&'a str);
impl<'a> QuadkeyRef<'a> {
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &str {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> &'a str {
        self.0
    }
}

impl<'a> QuadkeyRef<'a> {
    #[inline]
    #[must_use]
    pub fn new(s: &'a str) -> Option<Self> {
        is_quadkey(s).then(|| Self::new_unchecked(s))
    }

    /// Create a new `QuadkeyRef` if the given string is a valid quadkey.
    ///
    /// # Errors
    ///
    /// Returns `Err(QuadkeyError::InvalidLength)` if the string length is 30 or more.
    /// Returns `Err(QuadkeyError::InvalidChar)` if the string contains invalid characters.
    #[inline]
    pub fn try_new(s: &'a str) -> Result<Self, QuadkeyError> {
        if s.len() >= 30 {
            Err(QuadkeyError::InvalidLength(s.len()))
        } else if !is_quadkey(s) {
            Err(QuadkeyError::InvalidChar)
        } else {
            Ok(Self::new_unchecked(s))
        }
    }

    #[inline]
    #[must_use]
    pub fn new_unchecked(s: &'a str) -> Self {
        debug_assert!(is_quadkey(s));
        Self(s)
    }

    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'a str {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.0.as_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[expect(clippy::cast_possible_truncation)]
    #[inline]
    #[must_use]
    pub const fn zoom(&self) -> u8 {
        self.len() as u8
    }

    /// Return `(x, y, z)` tile coordinates
    #[inline]
    #[must_use]
    pub fn to_xyz_iter(&self) -> (u32, u32, u8) {
        let z = self.zoom();
        if z == 0 {
            return (0, 0, 0);
        }
        let mut x = 0;
        let mut y = 0;

        for (i, c) in self.0.bytes().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            let mask = 1 << (z - 1 - i as u8);
            match c {
                b'0' => {}
                b'1' => x |= mask,
                b'2' => y |= mask,
                b'3' => {
                    x |= mask;
                    y |= mask;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        (x, y, z)
    }

    #[inline]
    #[must_use]
    pub fn to_xyz(&self) -> (u32, u32, u8) {
        if self.zoom() == 0 {
            return (0, 0, 0);
        }
        self.0.bytes().map(|byte| u32::from(byte - b'0')).fold(
            (0, 0, 0),
            |(x, y, z), qk_int| {
                let xx = (x << 1) | (qk_int & 1);
                let yy = (y << 1) | (qk_int >> 1);
                let zz = z + 1;
                (xx, yy, zz)
            },
        )
    }

    #[inline]
    #[must_use]
    pub fn to_tile(&self) -> Tile {
        self.to_xyz().into()
    }
}

impl<'a> TryFrom<&'a str> for QuadkeyRef<'a> {
    type Error = QuadkeyError;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        QuadkeyRef::try_new(value)
    }
}
