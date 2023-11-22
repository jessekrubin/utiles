// converted by chadwick-general-purpose-tool from geostats schema.
// ref: from https://github.com/mapbox/mapbox-geostats
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileStats {
    pub layer_count: f64,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub layer: String,
    pub count: f64,
    pub geometry: GeometryType,
    pub attribute_count: f64,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryType {
    Point,
    LineString,
    Polygon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub attribute: String,
    pub count: f64,
    pub r#type: DataType,
    pub values: Vec<serde_json::Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Number,
    Boolean,
    Null,
    Mixed,
}
