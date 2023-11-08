use geo_types::coord;
use geo_types::Coord;
use geojson::{Geometry, Value as GeoJsonValue};
use std::iter::{Chain, Flatten, Once};
use std::vec::IntoIter;
use tracing::{warn, warn_span};

pub fn geojson_geometry_points(g: Geometry) -> Box<dyn Iterator<Item = Vec<f64>>> {
    let value = g.value;
    match value {
        GeoJsonValue::Point(c) => Box::new(std::iter::once(c)),
        GeoJsonValue::MultiPoint(points) => Box::new(points.into_iter()),
        GeoJsonValue::LineString(line_string) => Box::new(line_string.into_iter()),
        GeoJsonValue::MultiLineString(multi_line_string) => {
            Box::new(multi_line_string.into_iter().flatten())
        }
        GeoJsonValue::Polygon(polygon) => Box::new(polygon.into_iter().flatten()),
        GeoJsonValue::MultiPolygon(multi_polygon) => {
            Box::new(multi_polygon.into_iter().flatten().flatten())
        }
        GeoJsonValue::GeometryCollection(geometries) => {
            Box::new(geometries.into_iter().flat_map(geojson_geometry_points))
        }
        _ => Box::new(std::iter::empty()), // For any other case, return an empty iterator
    }
}

pub fn geojson_geometry_coords(g: Geometry) -> Box<dyn Iterator<Item = Coord>> {
    let coord_vecs = geojson_geometry_points(g);
    Box::new(coord_vecs.into_iter().map(|v| {
        coord! { x: v[0], y: v[1]}
    }))
}

pub fn geojson_geometry_points_vec(g: Geometry) -> Vec<Vec<f64>> {
    let value = g.value;
    let coord_vecs = match value {
        GeoJsonValue::Point(c) => {
            vec![c]
        }
        GeoJsonValue::MultiPoint(c) => c.into_iter().collect(),
        GeoJsonValue::LineString(c) => c.into_iter().collect(),
        GeoJsonValue::MultiLineString(c) => c.into_iter().flatten().collect(),
        GeoJsonValue::Polygon(c) => c.into_iter().flatten().collect(),
        GeoJsonValue::MultiPolygon(c) => c.into_iter().flatten().flatten().collect(),
        GeoJsonValue::GeometryCollection(c) => {
            let t = c
                .into_iter()
                .map(|g| geojson_geometry_points_vec(g))
                .flatten()
                .collect();
            t
        }
        _ => {
            vec![]
        }
    };
    coord_vecs
}
