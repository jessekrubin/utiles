use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Metadata row struct for `mbtiles` metadata table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRow {
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

impl MbtilesMetadataRow {
    /// Create a new `MbtilesMetadataRow`
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        MbtilesMetadataRow { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtilesMetadataRow>;

impl From<MbtilesMetadataRow> for MbtilesMetadataRowParsed {
    fn from(row: MbtilesMetadataRow) -> Self {
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
