use json_patch::Patch;
use serde::Serialize;
use serde_json::{Map, Value};
use tracing::warn;
use crate::utilesqlite::mbtiles::{metadata_delete, metadata_set};

#[derive(Debug, Serialize)]
pub struct MetadataChange {
    pub timestamp: String,
    pub forward: Patch,
    pub reverse: Patch,
    pub data: Value,
}


impl MetadataChange {
    pub fn from_forward_reverse_data(forward: Patch, reverse: Patch, data: Value) -> Self {
        MetadataChange {
            timestamp: chrono::Utc::now().to_rfc3339(),
            forward,
            reverse,
            data,
        }
    }
    pub fn forward_keys(&self) -> Vec<String> {
        self.forward.iter().map(|op| op.path().to_string()).map(
            // get only the first part
            |path| path.split('/').nth(1).unwrap().to_string()
        ).collect()
    }

    pub fn reverse_keys(&self) -> Vec<String> {
        self.reverse.iter().map(|op| op.path().to_string()).map(
            // get only the first part
            |path| path.split('/').nth(1).unwrap().to_string()
        ).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.forward.is_empty() && self.reverse.is_empty()
    }

    pub fn apply_changes_to_connection(
        conn: &rusqlite::Connection,
        changes: &Vec<MetadataChange>,
    ) -> Result<(), rusqlite::Error> {
        for change in changes {
            for patch in change.forward.iter() {
                match patch {
                    json_patch::PatchOperation::Add(_op) => {
                        let metadata_key = patch.path().to_string().split('/').nth(1).unwrap().to_string();
                        let value = change.data.get(&metadata_key).unwrap();
                        let value_string = serde_json::to_string(&value).unwrap();
                        metadata_set(conn, &metadata_key, &value_string)?;
                        // let sql = format!("INSERT INTO metadata (name, value) VALUES (?, ?)");
                        // conn.execute(&sql, &[&path, &value_string])?;
                    }
                    json_patch::PatchOperation::Replace(_op) => {
                        let metadata_key = patch.path().to_string().split('/').nth(1).unwrap().to_string();
                        let value = change.data.get(&metadata_key).unwrap();
                        let value_string = serde_json::to_string(&value).unwrap();
                        metadata_set(conn, &metadata_key, &value_string)?;
                    }
                    json_patch::PatchOperation::Remove(_op) => {
                        let metadata_key = patch.path().to_string().split('/').nth(1).unwrap().to_string();
                        metadata_delete(conn, &metadata_key)?;
                        // let path = op.path().to_string();
                        // let sql = format!("DELETE FROM metadata WHERE name = ?");
                        // conn.execute(&sql, &[&path])?;
                    }
                    _ => {
                        warn!("Unimplemented patch operation: {:?}", patch);
                    }
                }
            }
        }
        Ok(())
    }
}

