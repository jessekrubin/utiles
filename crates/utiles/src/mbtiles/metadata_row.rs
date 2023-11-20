use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRow {
    pub name: String,
    pub value: String,
}

pub type MbtilesMetadataRows = Vec<MbtilesMetadataRow>;
