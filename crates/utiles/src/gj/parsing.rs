use geojson::Position;
use serde_json::Value;
use utiles_core::{BBox, UtilesCoreError};

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::gj::geojson_coords;

pub fn parse_bbox_geojson(string: &str) -> UtilesResult<BBox> {
    // strip leading/trailing  whitespace
    let s = string.trim();
    // if the first char is "{" assume it is geojson-like
    if s.starts_with('{') {
        // parse to serde_json::Value
        let v: Value = serde_json::from_str(s)?;
        // if it has a "bbox" key, use that
        if v["bbox"].is_array() {
            let bbox: (f64, f64, f64, f64) = serde_json::from_value(v["bbox"].clone())
                .map_err(UtilesError::SerdeJsonError)?;
            return Ok(BBox::from(bbox));
        }
        return geojson_bounds(s);
    }
    let v: Value = serde_json::from_str(s)?;
    // Assume a single pair of coordinates represents a CoordTuple
    // and a four-element array represents a BBoxTuple
    match v.as_array().map(Vec::len) {
        // match len 0, 1, 3
        Some(0 | 1 | 3) => {
            Err(UtilesCoreError::InvalidBbox("Invalid bbox: ".to_string() + s).into())
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
            let bbox_arr = v.as_array();
            match bbox_arr {
                Some(arr) => {
                    let bbox = serde_json::from_value::<(f64, f64, f64, f64)>(
                        Value::Array(arr.iter().take(4).cloned().collect()),
                    )
                    .map_err(|e| UtilesError::ParsingError(e.to_string()))?;
                    Ok(BBox::from(bbox))
                }
                None => {
                    Err(UtilesError::ParsingError("Invalid bbox: ".to_string() + s))
                }
            }
        }
    }
}

pub fn coords2bounds<I>(mut coords: I) -> Option<(f64, f64, f64, f64)>
where
    I: Iterator<Item = Position>,
{
    // Initialize the bounds with the first coordinate.
    let first_coord = coords.next()?;
    let mut min_x = first_coord[0];
    let mut max_x = first_coord[0];
    let mut min_y = first_coord[1];
    let mut max_y = first_coord[1];

    // Iterate through the coordinates to find the extremes.
    for coord in coords {
        if coord[0] < min_x {
            min_x = coord[0];
        }
        if coord[0] > max_x {
            max_x = coord[0];
        }
        if coord[1] < min_y {
            min_y = coord[1];
        }
        if coord[1] > max_y {
            max_y = coord[1];
        }
    }

    Some((min_x, min_y, max_x, max_y))
}

pub fn geojson_bounds(geojson_str: &str) -> UtilesResult<BBox> {
    let coords = geojson_coords(geojson_str)?;
    let bounds = coords2bounds(coords);
    match bounds {
        Some(bounds) => Ok(BBox::new(bounds.0, bounds.1, bounds.2, bounds.3)),
        None => Err(UtilesError::ParsingError("Invalid bbox".to_string())),
    }
}
