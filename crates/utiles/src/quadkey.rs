use std::error::Error;
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

pub fn quadkey2xyz(quadkey: &str) -> Result<(u32, u32, u8), Box<dyn Error>> {
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
                Err("Invalid quadkey char found")?;
                // panic!("Invalid quadkey char: {}", c);
            }
        }
    }
    Ok((x, y, z))
}

pub fn quadkey2tile(quadkey: &str) -> Result<Tile, Box<dyn Error>> {
    let (x, y, z) = quadkey2xyz(quadkey)?;
    Ok(Tile::new(x, y, z))
}
