use serde::{Deserialize, Serialize};

#[allow(dead_code)]

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Coordinate2d(f64, f64);
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Coordinate3d(f64, f64, f64);
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Coordinate4d(f64, f64, f64, f64);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeojsonCoordinate {
    Coordinate2d(Coordinate2d),
    Coordinate3d(Coordinate3d),
}

pub type LineStringGeneric<T> = Vec<T>;
pub type LineString2d = Vec<Coordinate2d>;
pub type LineString3d = Vec<Coordinate3d>;
pub type LineString = Vec<GeojsonCoordinate>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PointGeometry {
    pub r#type: String,
    pub coordinates: GeojsonCoordinate,
}

pub type PointType = GeojsonCoordinate;
pub type MultiPointType = Vec<GeojsonCoordinate>;
pub type LineStringType = LineString;

pub enum GeometryTypeGeneric<TCoordinate> {
    Point(TCoordinate),
    MultiPoint(Vec<TCoordinate>),
    LineString(Vec<TCoordinate>),
    // MultiLineString (Vec<Vec<TCoordinate>>),
    // Polygon (Vec<Vec<TCoordinate>>),
    // MultiPolygon (Vec<Vec<Vec<TCoordinate>>>),
    // GeometryCollection (Vec<GeometryTypeGeneric<TCoordinate>>),
}
pub type Geometry2d = GeometryTypeGeneric<Coordinate2d>;
pub type Geometry3d = GeometryTypeGeneric<Coordinate3d>;
pub type Geometry = GeometryTypeGeneric<GeojsonCoordinate>;

// pub enum GeometryTypeGeneric < TCoordinate >

// > {
//     Point(PointType),
//     MultiPoint (MultiPointType),
//     LineString (LineStringType),
//     // MultiLineString (MultiLineStringType),
//     // Polygon (PolygonType),
//     // MultiPolygon,
//     // GeometryCollection,
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sometest() {
        assert_eq!(1, 1);

        let mixed = vec![
            GeojsonCoordinate::Coordinate2d(Coordinate2d(1.0, 2.0)),
            GeojsonCoordinate::Coordinate3d(Coordinate3d(3.0, 4.0, 5.0)),
        ];

        let string = serde_json::to_string(&mixed).unwrap();
        println!("{}", string);
        assert_eq!(string, "[[1.0,2.0],[3.0,4.0,5.0]]");

        let unstring: Vec<GeojsonCoordinate> = serde_json::from_str(&string).unwrap();
        println!("{:?}", unstring);
        assert_eq!(unstring, mixed);
    }
}
