use geo_types::Coord;
use serde_json::Value;

use utiles_core::bbox::BBox;
use utiles_core::errors::UtilesCoreResult;
use utiles_core::UtilesCoreError;

// use crate::geojson::geojson_bounds;
use crate::gj::geojson_coords;

pub fn parse_bbox_geojson(string: &str) -> UtilesCoreResult<BBox> {
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
