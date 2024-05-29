//! Geostats structs/models/objs
//!
//! ref: [mapbox/mapbox-geostats](https://github.com/mapbox/mapbox-geostats)
//!
//! Usually comes from tippecanoe's `json` metadata field from a mbtiles db.
//!
//! Converted w/ the help of chadwick-general-purpose-tool (`ChadGPT`) from geostats schema.
use serde::{Deserialize, Serialize};

/// `TileStats` struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileStats {
    /// Layer count
    pub layer_count: f64,

    /// Layers
    pub layers: Vec<Layer>,
}

/// Layer struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    /// Layer name
    pub layer: String,
    /// Feature count
    pub count: f64,
    /// Geometry type
    pub geometry: GeometryType,
    /// Attribute count
    pub attribute_count: f64,
    /// Attributes
    pub attributes: Vec<Attribute>,
}

/// `GeometryType` enum
#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryType {
    /// Point
    Point,
    /// `LineString`
    LineString,
    /// Polygon
    Polygon,
}

/// Attribute struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute name
    pub attribute: String,
    /// Value count
    pub count: f64,
    /// Data type
    pub r#type: DataType,
    /// Values
    pub values: Vec<serde_json::Value>,
    /// Min value
    pub min: Option<f64>,
    /// Max value
    pub max: Option<f64>,
}

/// Attribute `DataType` enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// String data type
    String,
    /// Number data type
    Number,
    /// Boolean data type
    Boolean,
    /// Null data type
    Null,
    /// Mixed data type
    Mixed,
}
