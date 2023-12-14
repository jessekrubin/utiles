use crate::errors::{UtilesError, UtilesResult};
use crate::Tile;

// tile = ut.Tile(486, 332, 10)
// expected = "0313102310"
/// Return the quadkey for a tile as a string.
/// # Examples
/// ```
/// use utiles::xyz2quadkey;
/// let quadkey = xyz2quadkey(486, 332, 10);
/// assert_eq!(quadkey, "0313102310");
/// ```
#[must_use]
pub fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    let mut quadkey = String::new();
    for i in (0..z).rev() {
        let mut digit = 0;
        let mask = 1 << i;
        if (x & mask) != 0 {
            digit += 1;
        }
        if (y & mask) != 0 {
            digit += 2;
        }
        quadkey.push_str(&digit.to_string());
    }
    quadkey
}

/// Return (x, y, z) for a quadkey as a tuple.
///
/// # Examples
/// ```
/// use utiles::quadkey2xyz;
/// let one_two_three = quadkey2xyz("123").unwrap();
/// assert_eq!(one_two_three, (5, 3, 3));
/// let xyz = quadkey2xyz("0313102310").unwrap();
/// assert_eq!(xyz, (486, 332, 10));
/// ```
pub fn quadkey2xyz(quadkey: &str) -> UtilesResult<(u32, u32, u8)> {
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
                return Err(UtilesError::InvalidQuadkey(c.to_string()));
            }
        }
    }
    Ok((x, y, z))
}

pub fn quadkey2tile(quadkey: &str) -> UtilesResult<Tile> {
    let xyz = quadkey2xyz(quadkey);
    match xyz {
        Ok((x, y, z)) => Ok(Tile::new(x, y, z)),
        Err(e) => Err(e),
    }
}
