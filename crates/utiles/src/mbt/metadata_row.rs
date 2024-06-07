use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Metadata row struct for `mbtiles` metadata table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtMetadataRow {
    /// name TEXT NOT NULL
    pub name: String,
    /// value TEXT NOT NULL
    pub value: String,
}

/// Metadata row struct for `mbtiles` metadata table with parsed values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRowParsed {
    /// name TEXT NOT NULL
    pub name: String,
    /// value TEXT NOT NULL
    pub value: Value,
}

impl MbtMetadataRow {
    /// Create a new `MbtilesMetadataRow`
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        MbtMetadataRow { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtMetadataRow>;

impl From<MbtMetadataRow> for MbtilesMetadataRowParsed {
    fn from(row: MbtMetadataRow) -> Self {
        let value = match row.value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(row.value.clone()),
        };
        MbtilesMetadataRowParsed {
            name: row.name,
            value,
        }
    }
}
