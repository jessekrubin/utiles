use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MbtilesMetadataRow {
    pub name: String,
    pub value: String,
}
