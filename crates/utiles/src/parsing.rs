use crate::bbox::BBox;
use crate::errors::UtilesResult;
use crate::geojson::geojson_coords;
use crate::UtilesError;
use geo_types::Coord;
use serde_json::Value;

pub fn parse_bbox_json(string: &str) -> UtilesResult<BBox> {
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
        return Ok(geojson_bounds(s));
    }

    let v: Value = serde_json::from_str(s)?;

    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    let bbox = match v.as_array().map(std::vec::Vec::len) {
        // match len 0, 1, 3
        Some(0) | Some(1) | Some(3) => {
            Err(UtilesError::InvalidBbox("Invalid bbox: ".to_string() + s))
        }
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
/// use utiles::parsing::parse_bbox;
/// let bbox = parse_bbox("-180,-85,180,85").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles::parsing::parse_bbox;
/// let bbox = parse_bbox("-180.0, -85.0, 180.0, 85.0").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles::parsing::parse_bbox;
/// let bbox = parse_bbox("-180.0 -85.0 180.0 85.0").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
///
/// ```
/// use utiles::parsing::parse_bbox;
/// let bbox = parse_bbox("[-180.0, -85.0, 180.0, 85.0]").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
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
        Ok(BBox::new(parts[0], parts[1], parts[2], parts[3]))
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
/// use utiles::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("world").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -90.0, 180.0, 90.0));
/// ```
///
/// ```
/// use utiles::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("planet").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -90.0, 180.0, 90.0));
/// ```
///
/// ```
/// use utiles::parsing::parse_bbox_ext;
/// let bbox = parse_bbox_ext("-180,-85,180,85").unwrap();
/// assert_eq!(bbox, utiles::bbox::BBox::new(-180.0, -85.0, 180.0, 85.0));
/// ```
pub fn parse_bbox_ext(string: &str) -> Result<BBox, String> {
    // match 'world' or 'planet'
    if string == "world" || string == "planet" {
        return Ok(BBox::new(-180.0, -90.0, 180.0, 90.0));
    }
    // trim leading/trailing single/double quotes
    let string = string.trim_matches(|c| c == '\'' || c == '"');
    parse_bbox(string)
}

pub fn coords2bounds<I>(mut coords: I) -> Option<(f64, f64, f64, f64)>
where
    I: Iterator<Item = Coord>,
{
    // Initialize the bounds with the first coordinate.
    let first_coord = coords.next()?;
    let mut min_x = first_coord.x;
    let mut max_x = first_coord.x;
    let mut min_y = first_coord.y;
    let mut max_y = first_coord.y;

    // Iterate through the coordinates to find the extremes.
    for coord in coords {
        if coord.x < min_x {
            min_x = coord.x;
        }
        if coord.x > max_x {
            max_x = coord.x;
        }
        if coord.y < min_y {
            min_y = coord.y;
        }
        if coord.y > max_y {
            max_y = coord.y;
        }
    }

    Some((min_x, min_y, max_x, max_y))
}

#[must_use]
pub fn geojson_bounds(geojson_str: &str) -> BBox {
    let coords = geojson_coords(geojson_str);
    let bounds = coords2bounds(coords).unwrap();
    BBox::new(bounds.0, bounds.1, bounds.2, bounds.3)
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
