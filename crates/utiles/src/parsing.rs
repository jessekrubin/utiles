use crate::bbox::BBox;
use crate::geojson::geojson_coords;
use geo_types::Coord;
use serde_json::Value;
use tracing::debug;

// pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
    // if the first char is "{" assume it is geojson-like
    debug!("parse_bbox: {}", s);
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

    debug!("{}", v);
    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    let bbox = match v.as_array().map(|arr| arr.len()) {
        Some(2) => {
            let coord: (f64, f64) = serde_json::from_value::<(f64, f64)>(v)?;
            Ok(BBox::new(coord.0, coord.1, coord.0, coord.1))
        }
        Some(4) => {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::from(bbox))
        }
        _ => panic!("Expected a two-element array or a four-element array"),
    };
    debug!("bbox: {:?}", bbox);
    bbox
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
}
