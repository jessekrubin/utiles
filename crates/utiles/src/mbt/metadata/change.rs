use crate::utilesqlite::mbtiles::{metadata_delete, metadata_set};
use json_patch::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

// #[derive(Debug, Serialize, strum_macros::EnumString)]
// #[strum(serialize_all = "snake_case")]
// pub enum DbChangeType {
//     Metadata,
//     Unknown,
// }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MetadataChangeFromTo {
    pub name: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MetadataChange {
    pub changes: Vec<MetadataChangeFromTo>,
    pub forward: Patch,
    pub reverse: Patch,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum DbChangeType {
    Metadata(MetadataChange),
    Unknown(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DbChangeset {
    pub timestamp: String,
    pub changes: Vec<DbChangeType>,
}

impl From<MetadataChange> for DbChangeType {
    fn from(change: MetadataChange) -> Self {
        Self::Metadata(change)
    }
}

impl From<DbChangeType> for DbChangeset {
    fn from(change: DbChangeType) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            changes: vec![change],
        }
    }
}

impl MetadataChange {
    #[must_use]
    pub fn new_empty() -> Self {
        Self {
            changes: vec![],
            forward: Patch(vec![]),
            reverse: Patch(vec![]),
            data: Value::Null,
        }
    }

    #[must_use]
    pub fn forward_keys(&self) -> Vec<String> {
        self.forward
            .iter()
            .map(|op| op.path().to_string())
            .filter_map(|path| path.split('/').nth(1).map(|s| s.to_string()))
            .map(|s| s.to_string())
            .collect()
    }

    #[must_use]
    pub fn reverse_keys(&self) -> Vec<String> {
        self.reverse
            .iter()
            .map(|op| op.path().to_string())
            .filter_map(|path| path.split('/').nth(1).map(|s| s.to_string()))
            .map(|s| s.to_string())
            .collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty() && self.reverse.is_empty()
    }

    pub fn apply_changes_to_connection(
        conn: &rusqlite::Connection,
        changes: &Vec<MetadataChange>,
    ) -> Result<(), rusqlite::Error> {
        for change in changes {
            for patch in change.forward.iter() {
                let path_string = patch.path().to_string();

                let metadata_key = path_string.split('/').nth(1);
                match metadata_key {
                    Some(metadata_key) => match patch {
                        json_patch::PatchOperation::Add(_op) => {
                            if let Some(value) = change.data.get(metadata_key) {
                                let value_string = match value {
                                    Value::String(s) => Ok(s.to_string()),
                                    _ => serde_json::to_string(&value),
                                };
                                if let Ok(value_string) = value_string {
                                    metadata_set(conn, metadata_key, &value_string)?;
                                } else {
                                    warn!("Failed to serialize value: {:?}", value);
                                }
                            }
                        }
                        json_patch::PatchOperation::Replace(_op) => {
                            if let Some(value) = change.data.get(metadata_key) {
                                let value_string = match value {
                                    Value::String(s) => Ok(s.to_string()),
                                    _ => serde_json::to_string(&value),
                                };
                                if let Ok(value_string) = value_string {
                                    metadata_set(conn, metadata_key, &value_string)?;
                                } else {
                                    warn!("Failed to serialize value: {:?}", value);
                                }
                            }
                        }
                        json_patch::PatchOperation::Remove(_op) => {
                            metadata_delete(conn, metadata_key)?;
                        }
                        _ => {
                            warn!("Unimplemented patch operation: {:?}", patch);
                        }
                    },
                    None => {
                        warn!("metadata_key is None");
                    }
                }
                // match patch {
                //     json_patch::PatchOperation::Add(_op) => {
                //         let value = change.data.get(&metadata_key).unwrap();
                //         let value_string = serde_json::to_string(&value).unwrap();
                //         metadata_set(conn, &metadata_key, &value_string)?;
                //         // let sql = format!("INSERT INTO metadata (name, value) VALUES (?, ?)");
                //         // conn.execute(&sql, &[&path, &value_string])?;
                //     }
                //     json_patch::PatchOperation::Replace(_op) => {
                //         let metadata_key = patch
                //             .path()
                //             .to_string()
                //             .split('/')
                //             .nth(1)
                //             .unwrap()
                //             .to_string();
                //         let value = change.data.get(&metadata_key).unwrap();
                //         let value_string = serde_json::to_string(&value).unwrap();
                //         metadata_set(conn, &metadata_key, &value_string)?;
                //     }
                //     json_patch::PatchOperation::Remove(_op) => {
                //         let metadata_key = patch
                //             .path()
                //             .to_string()
                //             .split('/')
                //             .nth(1)
                //             .unwrap()
                //             .to_string();
                //         metadata_delete(conn, &metadata_key)?;
                //         // let path = op.path().to_string();
                //         // let sql = format!("DELETE FROM metadata WHERE name = ?");
                //         // conn.execute(&sql, &[&path])?;
                //     }
                //     _ => {
                //         warn!("Unimplemented patch operation: {:?}", patch);
                //     }
                // }
            }
        }
        Ok(())
    }
}
