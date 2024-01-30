use serde_json::Value;

use crate::bbox::BBox;
use crate::errors::UtilesCoreResult;
use crate::UtilesCoreError;

pub fn parse_bbox_json(string: &str) -> UtilesCoreResult<BBox> {
    // strip leading/trailing  whitespace
    let s = string.trim();
    // if the first char is "{" assume it is geojson-like
    if s.starts_with('{') {
        // parse to serde_json::Value
        let v: Value = serde_json::from_str(s)?;
        // if it has a "bbox" key, use that
        if v["bbox"].is_array() {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v["bbox"].clone())?;
            return Ok(BBox::from(bbox));
        }
        // return Ok(geojson_bounds(s));

        return Err(UtilesCoreError::InvalidBbox(
            "Invalid bbox: ".to_string() + s,
        ));
    }

    let v: Value = serde_json::from_str(s)?;

    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    let bbox = match v.as_array().map(std::vec::Vec::len) {
        // match len 0, 1, 3
        Some(0) | Some(1) | Some(3) => Err(UtilesCoreError::InvalidBbox(
            "Invalid bbox: ".to_string() + s,
        )),
        Some(2) => {
            let coord: (f64, f64) = serde_json::from_value::<(f64, f64)>(v)?;
            Ok(BBox::new(coord.0, coord.1, coord.0, coord.1))
        }
        Some(4) => {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::from(bbox))
        }
        _ => {
            // take first four elements
            let bbox_vec = v
                .as_array()
                .unwrap()
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<Value>>();
            Ok(BBox::from(serde_json::from_value::<(f64, f64, f64, f64)>(
                Value::Array(bbox_vec),
            )?))
        }
    };
    bbox
}

/// Parse a string into a BBox
///
/// # Examples
///
/// ```
/// use utiles_core::parsing::parse_bbox;
/// let bbox = parse_bbox("-180,-85,180,85").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles_core::parsing::parse_bbox;
/// let bbox = parse_bbox("-180.0, -85.0, 180.0, 85.0").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles_core::parsing::parse_bbox;
/// let bbox = parse_bbox("-180.0 -85.0 180.0 85.0").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles_core::parsing::parse_bbox;
/// let bbox = parse_bbox("[-180.0, -85.0, 180.0, 85.0]").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
pub fn parse_bbox(string: &str) -> Result<BBox, String> {
    // strip leading/trailing  whitespace
    let s = string.trim();
    // if the first char is "{" assume it is geojson-like
    if s.starts_with('{') || s.starts_with('[') {
        let bbox = parse_bbox_json(s);
        return bbox.map_err(|e| e.to_string());
    }
    let parts: Vec<f64> = if s.contains(',') {
        s.split(',')
            .map(|p| p.trim())
            .filter_map(|p| p.parse::<f64>().ok())
            .collect()
    } else if s.contains(' ') {
        s.split(' ')
            .map(|p| p.trim())
            .filter_map(|p| p.parse::<f64>().ok())
            .collect()
    } else {
        vec![]
    };
    if parts.len() == 4 {
        // if north < south err out
        if parts[3] < parts[1] {
            Err("Invalid bbox: ".to_string() + s + " (north < south)")
        } else {
            Ok(BBox::new(parts[0], parts[1], parts[2], parts[3]))
        }
    } else {
        let msg = format!("Invalid bbox: {s}");
        Err(msg)
    }
}

/// Parse a string into a BBox with special handling of 'world' and 'planet'
///
/// # Examples
///
/// ```
/// use utiles_core::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("world").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -90.0, 180.0, 90.0));
/// ```
///
/// ```
/// use utiles_core::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("planet").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -90.0, 180.0, 90.0));
/// ```
///
/// ```
/// use utiles_core::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("-180,-85,180,85").unwrap();
/// assert_eq!(bbox, utiles_core::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
pub fn parse_bbox_ext(string: &str) -> Result<BBox, String> {
    // match 'world' or 'planet'
    // match string/lower
    let str_lower = string.to_lowercase();
    let r = match str_lower.as_str() {
        "world" | "planet" | "all" | "*" => Ok(BBox::new(-180.0, -90.0, 180.0, 90.0)),
        "n" | "north" => Ok(BBox::new(-180.0, 0.0, 180.0, 90.0)),
        "s" | "south" => Ok(BBox::new(-180.0, -90.0, 180.0, 0.0)),
        "e" | "east" => Ok(BBox::new(0.0, -90.0, 180.0, 90.0)),
        "w" | "west" => Ok(BBox::new(-180.0, -90.0, 0.0, 90.0)),
        "ne" | "northeast" => Ok(BBox::new(0.0, 0.0, 180.0, 90.0)),
        "nw" | "northwest" => Ok(BBox::new(-180.0, 0.0, 0.0, 90.0)),
        "se" | "southeast" => Ok(BBox::new(0.0, -90.0, 180.0, 0.0)),
        "sw" | "southwest" => Ok(BBox::new(-180.0, -90.0, 0.0, 0.0)),
        _ => parse_bbox(string),
    };
    r
}

#[must_use]
pub fn string_is_digits(string: &str) -> bool {
    string.chars().all(|c| c.is_ascii_digit())
}

/// Parse a string into vector of integer strings
///
/// # Examples
/// ```
/// use utiles_core::parsing::parse_uint_strings;
/// let ints = parse_uint_strings("1,2,3,4,5");
/// assert_eq!(ints, vec!["1", "2", "3", "4", "5"]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_uint_strings;
/// let ints = parse_uint_strings("x1y2z3");
/// assert_eq!(ints, vec!["1", "2", "3"]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_uint_strings;
/// let ints = parse_uint_strings("as;ldfkjas;ldfkj");
/// assert_eq!(ints, Vec::<String>::new());
/// ```
///
/// ```
/// use utiles_core::parsing::parse_uint_strings;
/// let ints = parse_uint_strings("http://example.com/tiles/3/2/1.png");
/// assert_eq!(ints, vec!["3", "2", "1"]);
/// ```
#[must_use]
pub fn parse_uint_strings(input: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = None;
    for (i, c) in input.char_indices() {
        if c.is_ascii_digit() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            blocks.push(&input[s..i]);
            start = None;
        }
    }
    if let Some(s) = start {
        blocks.push(&input[s..]);
    }
    blocks
}

/// Parse a string into vector of integers
///
/// # Examples
/// ```
/// use utiles_core::parsing::parse_uints;
/// let ints = parse_uints("1,2,3,4,5");
/// assert_eq!(ints, vec![1, 2, 3, 4, 5]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_uints;
/// let ints = parse_uints("x1y2z3");
/// assert_eq!(ints, vec![1, 2, 3]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_uints;
/// let ints = parse_uints("as;ldfkjas;ldfkj");
/// assert_eq!(ints, Vec::<u64>::new());
/// ```
#[must_use]
pub fn parse_uints(input: &str) -> Vec<u64> {
    parse_uint_strings(input)
        .iter()
        .map(|s| s.parse::<u64>().unwrap())
        .collect()
}

/// Parse a string into a vector of signed integer strings
///
/// # Examples
/// ```
/// use utiles_core::parsing::parse_int_strings;
/// let ints = parse_int_strings("-1,2,---3,4,-5");
/// assert_eq!(ints, vec!["-1", "2", "-3", "4", "-5"]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_int_strings;
/// let ints = parse_int_strings("x-1y2z-3");
/// assert_eq!(ints, vec!["-1", "2", "-3"]);
/// ```
///
/// ```
/// use utiles_core::parsing::parse_int_strings;
/// let ints = parse_int_strings("as;ldfkjas;ldfkj");
/// assert_eq!(ints, Vec::<&str>::new());
/// ```
///
/// ```
/// use utiles_core::parsing::parse_int_strings;
/// let ints = parse_int_strings("http://example.com/tiles/-3/2/1.png");
/// assert_eq!(ints, vec!["-3", "2", "1"]);
/// ```
#[must_use]
pub fn parse_int_strings(input: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = None;
    let mut is_negative = false; // flag to track if the current number is negative

    for (i, c) in input.char_indices() {
        match c {
            '-' => {
                // If we encounter a '-' and no number has started, note the start and set negative flag
                if start.is_none() {
                    start = Some(i);
                    is_negative = true; // Expecting a number after this
                } else if let Some(s) = start {
                    // If '-' follows digits or another '-', end the previous block (if valid) and start a new one
                    if !is_negative || s < i - 1 {
                        // Check if previous char was also '-' or if it's a valid number
                        blocks.push(&input[s..i]);
                    }
                    start = Some(i);
                    is_negative = true;
                }
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                if start.is_none() || is_negative {
                    // Start of a new number block (potentially negative)
                    start = Some(i - is_negative as usize); // Include '-' in block if negative
                }
                is_negative = false; // Once we have digits, it's no longer just a '-'
            }
            _ => {
                // For any other character, end the current number block if it exists and is valid
                if let Some(s) = start {
                    if !is_negative || s < i - 1 {
                        // Ensure it's not just a '-' without digits
                        blocks.push(&input[s..i]);
                    }
                }
                // Reset for the next number
                start = None;
                is_negative = false;
            }
        }
    }
    // Capture the last number block if there's one and it's valid
    if let Some(s) = start {
        if !is_negative || s < input.len() - 1 {
            // Ensure it's not just a '-' without digits
            blocks.push(&input[s..]);
        }
    }
    blocks
}

/// Parse a string into a vector of signed integers
///
/// # Examples
/// ```
/// use utiles_core::parsing::parse_ints;
/// let ints = parse_ints("-1,2,---3,4,-5");
/// assert_eq!(ints, vec![-1, 2, -3, 4, -5]);
/// ```
#[must_use]
pub fn parse_ints(input: &str) -> Vec<i64> {
    parse_int_strings(input)
        .iter()
        .map(|s| s.parse::<i64>().unwrap())
        .collect()
}

/// Parse float string blocks from a string
///
/// # Examples
/// ```
/// use utiles_core::parsing::parse_float_blocks;
/// let input = "-123.45..6abc--7.8.9";
/// let blocks = parse_float_blocks(input);
/// assert_eq!(blocks, vec!["-123.45", ".6", "-7.8", ".9"]);
/// ```
///
#[must_use]
pub fn parse_float_blocks(input: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = None; // Start index of the current number block
    let mut has_decimal = false; // Track if the current block has a decimal point
    let mut has_digit = false; // Ensure there's at least one digit

    for (i, c) in input.char_indices() {
        match c {
            '0'..='9' => {
                if start.is_none() {
                    start = Some(i);
                }
                has_digit = true;
            }
            '-' => {
                if start.is_none() && !has_digit {
                    // Start of a new number
                    start = Some(i);
                } else if has_digit || has_decimal || start.is_some() {
                    // Malformed if in the middle of a number
                    if let Some(s) = start {
                        if has_digit {
                            // Ensure there's at least one digit
                            blocks.push(&input[s..i]);
                        }
                    }
                    start = Some(i); // Reset for a new potential number
                    has_decimal = false;
                    has_digit = false;
                }
            }
            '.' => {
                if !has_decimal && start.is_none() {
                    // First decimal in a new number
                    start = Some(i);
                    has_decimal = true;
                } else if has_decimal || start.is_none() {
                    // Malformed if another decimal or no start
                    if let Some(s) = start {
                        if has_digit {
                            // Ensure there's at least one digit
                            blocks.push(&input[s..i]);
                        }
                    }
                    start = Some(i); // Start a new potential number
                    has_decimal = true; // Current char is '.'
                    has_digit = false;
                } else {
                    // First decimal in an ongoing number
                    has_decimal = true;
                }
            }
            _ => {
                if let Some(s) = start {
                    if has_digit {
                        // Ensure there's at least one digit
                        blocks.push(&input[s..i]);
                    }
                }
                // Reset for the next number
                start = None;
                has_decimal = false;
                has_digit = false;
            }
        }
    }

    // Handle the last block if it's well-formed
    if let Some(s) = start {
        if has_digit {
            // Ensure there's at least one digit
            blocks.push(&input[s..]);
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use crate::bbox::*;
    use crate::parsing::parse_bbox;

    #[test]
    fn parse_bbox_simple() {
        let string = r#"[-180.0, -85.0, 180.0, 85.0]"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    #[test]
    fn parse_bbox_simple_len_5() {
        let string = r#"[-180.0, -85.0, 180.0, 85.0, "uhhhhh"]"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    #[test]
    fn parse_bbox_simple_len_6() {
        let string = r#"[-180.0, -85.0, 180.0, 85.0, 0, 10]"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    #[test]
    fn parse_bbox_str_commas() {
        let string = r#"-180.0, -85.0, 180.0, 85.0"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    #[test]
    fn parse_bbox_str_spaces() {
        let string = r#"-180.0 -85.0 180.0 85.0"#;
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, 180.0, 85.0));
    }

    #[test]
    fn parse_bbox_from_coords() {
        let string = "[-180.0, -85.0]";
        let bbox_result = parse_bbox(string);
        // assert!(bbox_result.is_ok());
        let bbox = bbox_result.unwrap();
        assert_eq!(bbox, BBox::new(-180.0, -85.0, -180.0, -85.0));
    }

    #[test]
    fn parse_bbox_bad() {
        let string = r#"[-180.0,]"#;
        let bbox_result = parse_bbox(string);
        assert!(bbox_result.is_err());
    }

    #[test]
    fn parse_bbox_metadata_string() {
        let s = "-176.696694,-14.373776,145.830505,71.341324";
        let bbox = parse_bbox(s);
        assert!(bbox.is_ok());
        let bbox = bbox.unwrap();
        assert_eq!(
            bbox,
            BBox::new(-176.696_694, -14.373_776, 145.830_505, 71.341_324)
        );
    }
}
