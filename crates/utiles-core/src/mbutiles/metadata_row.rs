use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRow {
    pub name: String,
    pub value: String,
}

impl MbtilesMetadataRow {
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        MbtilesMetadataRow { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtilesMetadataRow>;
