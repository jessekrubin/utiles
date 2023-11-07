use crate::bbox::BBox;
use serde_json::Value;

// pub fn parse_bbox(s: &str) -> Result<BBox> {
//     let parsed: Result<BBoxParseAble> = serde_json::from_str(s);
//     let bbox = match parsed? {
//         BBoxParseAble::CoordTuple(coord) => BBox::new(coord.0, coord.1, coord.0, coord.1),
//         BBoxParseAble::BBoxTuple(bbox) => BBox::from(bbox),
//         // Uncomment and handle BBoxParseAble::Array(array) if needed
//         // BBoxParseAble::Array(array) => BBox::new(array[0], array[1], array[2], array[3]),
//     };
//     Ok(bbox)
// }
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
        _ => (panic!("Expected a two-element array or a four-element array")),
    }
}

//
// pub fn parse_bbox(s: &str) -> Result<BBox> {
//     let parsed : Result<BBoxParseAble>= serde_json::from_str(&s);
//     if parsed.is_err() {
//         // println!("parsed error: {:?}", parsed.err().unwrap());
//         return Err(parsed.err().unwrap())
//     }
//     let parsed = parsed.unwrap();
//     let bbox = match parsed {
//         BBoxParseAble::CoordTuple(coord) => {
//             let bbox = BBox::new(coord.0, coord.1, coord.0, coord.1);
//             bbox
//         },
//         BBoxParseAble::BBoxTuple(bbox) => {
//             let bbox = BBox::from(bbox);
//             bbox
//         },
//         // BBoxParseAble::Array(array) => {
//         //     let bbox = BBox::new(array[0], array[1], array[2], array[3]);
//         //     bbox
//         // },
//     };
//     return Ok(bbox);
//
// }
#[cfg(test)]
mod tests {
    use crate::bbox::*;
    use crate::parsing::parse_bbox;

    //
    // #[test]
    // fn parse_bbox_simple(){
    //     let string = "[-180.0, -85, 180.0, 85]";
    //     let bbox_result = parse_bbox(string);
    //     assert!(bbox_result.is_ok());
    //     let bbox = bbox_result.unwrap();
    //     assert_eq!(bbox, BBox::new( -180.0, -85.0, 180.0, 85.0));
    //
    // }

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
    //     let string = "[1, 2]";
    //     let bbox = parse_bbox(string).unwrap();
    //     assert_eq!(bbox, BBox::new(1.0, 2.0, 1.0, 2.0));
    // }
}
