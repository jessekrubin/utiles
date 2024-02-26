use serde::{Deserialize, Serialize};

/// Metadata row struct for `mbtiles` metadata table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRow {
    /// name TEXT NOT NULL
    pub name: String,
    /// value TEXT NOT NULL
    pub value: String,
}

impl MbtilesMetadataRow {
    /// Create a new MbtilesMetadataRow
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        MbtilesMetadataRow { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtilesMetadataRow>;
