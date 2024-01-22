use crate::errors::{UtilesCoreError, UtilesCoreResult};
use crate::Tile;
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
// tile = ut.Tile(486, 332, 10)
// expected = "0313102310"
/// Return the quadkey for a tile as a string.
/// # Examples
/// ```
/// use utiles_core::xyz2quadkey;
/// let quadkey = xyz2quadkey(486, 332, 10);
/// assert_eq!(quadkey, "0313102310");
/// ```
#[must_use]
pub fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    xyz2quadkey_vec(x, y, z)
        .iter()
        .map(|&c| (c + b'0') as char)
        .collect()
}

/// Return (x, y, z) for a quadkey as a tuple.
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
