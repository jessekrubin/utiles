use geo_types::{Coord};
use crate::bbox::BBox;
use serde_json::Value;
use geojson::{GeoJson, Geometry, Value as GeoJsonValue, Feature};

use geo_types::coord;

// pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
pub fn parse_bbox(s: &str) -> serde_json::Result<BBox> {
    // if the first char is "{" assume it is geojson-like
    if s.chars().next().unwrap() == '{' {
        let coords = geojson_coords(s);
        return Ok(BBox::from(coords));
    }

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


fn vec2coord(v: Vec<f64>) -> Coord {
    coord! { x: v[0], y: v[1]}
}

fn geojson_geometry_points(g: Geometry) -> Vec<Vec<f64>> {
    let value = g.value;
    let coord_vecs = match value {
        GeoJsonValue::Point(c) => {
            vec![c]
        }
        GeoJsonValue::MultiPoint(c) => {
            c.into_iter().collect()
        }
        GeoJsonValue::LineString(c) => {
            c.into_iter().collect()
        }
        GeoJsonValue::MultiLineString(c) => {
            c.into_iter().flatten().collect()
        }
        GeoJsonValue::Polygon(c) => {
            c.into_iter().flatten().collect()
        }
        GeoJsonValue::MultiPolygon(c) => {
            c.into_iter().flatten().flatten().collect()
        }
        GeoJsonValue::GeometryCollection(c) => {
            let t = c.into_iter().map(|g| geojson_geometry_points(g)).flatten().collect();
            t


        }
        _ => {
            vec![]
        }


    };
    coord_vecs


    // convert from Vec<f64> to Vec<Coord>
    // coord_vecs.into_iter().map(|v| vec2coord(v)).collect()

}
fn geojson_geometry_coords(g: Geometry) -> Vec<Coord> {
    let coord_vecs = geojson_geometry_points(g);
    // convert from Vec<f64> to Vec<Coord>
    coord_vecs.into_iter().map(|v| vec2coord(v)).collect()

}

fn geojson_feature_coords(feature: Feature) -> Vec<Coord> {
    let geometry = feature.geometry.unwrap();
    geojson_geometry_coords(geometry)
}

pub fn geojson_coords(geojson_str: &str) -> Vec<Coord> {
    let gj = geojson_str.parse::<GeoJson>().unwrap();
    match gj {
        GeoJson::FeatureCollection(fc) => {
            let mut coords = Vec::new();
            for feature in fc.features {
                let feature_coords = geojson_feature_coords(feature);
                coords.extend(feature_coords);
            }
            coords
            // let mut bbox = BBox::new(180.0, 90.0, -180.0, -90.0);
            // for feature in fc.features {
            //     let feature_bbox = geojson_feature_bounds(feature);
            //     bbox = bbox.union(feature_bbox);
            // }
            // bbox
        }
        GeoJson::Feature(feature) => {
            // if it has a bbox
            let geometry = feature.geometry.unwrap();
            geojson_geometry_coords(geometry)
        }
        GeoJson::Geometry(geometry) => {
            geojson_geometry_coords(geometry)
        }
    }
}

pub fn geojson_bounds(geojson_str: &str) -> BBox {
    let coords = geojson_coords(geojson_str);
    // BBox::from(coords)
    BBox::world_web()
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

