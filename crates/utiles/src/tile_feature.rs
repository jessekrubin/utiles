use crate::tile::TileFeatureGeometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileFeature {
    pub id: String,

    #[serde(rename = "type")]
    pub type_: String,

    pub geometry: TileFeatureGeometry,
    pub bbox: (f64, f64, f64, f64),
    pub properties: Map<String, Value>,
}

impl TileFeature {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn bbox_lons(&self) -> Vec<f64> {
        vec![self.bbox.0, self.bbox.2]
    }

    pub fn bbox_lats(&self) -> Vec<f64> {
        vec![self.bbox.1, self.bbox.3]
    }

    pub fn extents_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.bbox.0, self.bbox.1, self.bbox.2, self.bbox.3
        )
    }

    pub fn bbox_json(&self) -> String {
        format!(
            "[{},{},{},{}]",
            self.bbox.0, self.bbox.1, self.bbox.2, self.bbox.3
        )
    }
}
