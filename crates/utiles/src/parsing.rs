use crate::bbox::BBox;
use serde_json::Value;

// pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
    let v: Value = serde_json::from_str(s)?;

    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    match v.as_array().map(|arr| arr.len()) {
        Some(2) => {
            let coord: (f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::new(coord.0, coord.1, coord.0, coord.1))
        }
        Some(4) => {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v)?;
            Ok(BBox::from(bbox))
        }
        _ => panic!("Expected a two-element array or a four-element array"),
    }
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
