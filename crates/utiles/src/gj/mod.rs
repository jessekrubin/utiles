use crate::UtilesError;
use crate::errors::UtilesResult;
use geojson::{Feature, GeoJson, Geometry, GeometryValue, Position};

pub mod parsing;

pub fn geojson_geometry_points(g: Geometry) -> Box<dyn Iterator<Item = Position>> {
    match g.value {
        GeometryValue::Point { coordinates } => Box::new(std::iter::once(coordinates)),
        GeometryValue::MultiPoint { coordinates }
        | GeometryValue::LineString { coordinates } => {
            Box::new(coordinates.into_iter())
        }
        GeometryValue::MultiLineString { coordinates }
        | GeometryValue::Polygon { coordinates } => {
            Box::new(coordinates.into_iter().flatten())
        }
        GeometryValue::MultiPolygon { coordinates } => {
            Box::new(coordinates.into_iter().flatten().flatten())
        }
        GeometryValue::GeometryCollection { geometries } => {
            Box::new(geometries.into_iter().flat_map(geojson_geometry_points))
        }
    }
}

#[must_use]
pub fn geojson_geometry_coords(g: Geometry) -> Box<dyn Iterator<Item = Position>> {
    let coord_vecs = geojson_geometry_points(g);
    Box::new(coord_vecs.into_iter())
}

pub fn geojson_geometry_points_vec(g: Geometry) -> Vec<Position> {
    match g.value {
        GeometryValue::Point { coordinates } => vec![coordinates],
        GeometryValue::MultiPoint { coordinates }
        | GeometryValue::LineString { coordinates } => {
            coordinates.into_iter().collect()
        }
        GeometryValue::MultiLineString { coordinates }
        | GeometryValue::Polygon { coordinates } => {
            coordinates.into_iter().flatten().collect()
        }
        GeometryValue::MultiPolygon { coordinates } => {
            coordinates.into_iter().flatten().flatten().collect()
        }
        GeometryValue::GeometryCollection { geometries } => geometries
            .into_iter()
            .flat_map(geojson_geometry_points_vec)
            .collect(),
    }
}

#[must_use]
pub fn geojson_feature_coords(feature: Feature) -> Box<dyn Iterator<Item = Position>> {
    match feature.geometry {
        Some(g) => geojson_geometry_coords(g),
        None => Box::new(std::iter::empty()),
    }
}

pub fn geojson_coords(
    geojson_str: &str,
) -> UtilesResult<Box<dyn Iterator<Item = Position>>> {
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
