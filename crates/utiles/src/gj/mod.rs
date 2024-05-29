use crate::errors::UtilesResult;
use crate::UtilesError;
use geo_types::coord;
use geo_types::Coord;
use geojson::{Feature, GeoJson, Geometry, Value as GeoJsonValue};

pub mod parsing;

pub fn geojson_geometry_points(g: Geometry) -> Box<dyn Iterator<Item = Vec<f64>>> {
    match g.value {
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
    }
}

#[must_use]
pub fn geojson_geometry_coords(g: Geometry) -> Box<dyn Iterator<Item = Coord>> {
    let coord_vecs = geojson_geometry_points(g);
    Box::new(coord_vecs.into_iter().map(|v| {
        coord! { x: v[0], y: v[1]}
    }))
}

pub fn geojson_geometry_points_vec(g: Geometry) -> Vec<Vec<f64>> {
    match g.value {
        GeoJsonValue::Point(c) => vec![c],
        GeoJsonValue::MultiPoint(c) | GeoJsonValue::LineString(c) => {
            c.into_iter().collect()
        }
        GeoJsonValue::MultiLineString(c) | GeoJsonValue::Polygon(c) => {
            c.into_iter().flatten().collect()
        }
        GeoJsonValue::MultiPolygon(c) => c.into_iter().flatten().flatten().collect(),
        GeoJsonValue::GeometryCollection(c) => c
            .into_iter()
            .flat_map(geojson_geometry_points_vec)
            .collect(),
    }
}

#[must_use]
pub fn geojson_feature_coords(feature: Feature) -> Box<dyn Iterator<Item = Coord>> {
    match feature.geometry {
        Some(g) => geojson_geometry_coords(g),
        None => Box::new(std::iter::empty()),
    }
}

pub fn geojson_coords(
    geojson_str: &str,
) -> UtilesResult<Box<dyn Iterator<Item = Coord>>> {
    let gj = geojson_str
        .parse::<GeoJson>()
        .map_err(|e| UtilesError::ParsingError(e.to_string()))?;
    match gj {
        GeoJson::FeatureCollection(fc) => Ok(Box::new(
            fc.features.into_iter().flat_map(geojson_feature_coords),
        )),
        GeoJson::Feature(feature) => {
            // if it has a bbox
            let g = feature.geometry;
            match g {
                Some(g) => Ok(geojson_geometry_coords(g)),
                None => Err(UtilesError::ParsingError(
                    "Feature has no geometry".to_string(),
                )),
            }
        }
        GeoJson::Geometry(geometry) => Ok(geojson_geometry_coords(geometry)),
    }
}
