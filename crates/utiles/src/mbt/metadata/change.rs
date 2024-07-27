use json_patch::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::mbt::mbtiles::{metadata_delete, metadata_set};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MetadataChangeFromTo {
    pub name: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn escape_sql_string(value: &str) -> String {
    value.replace("'", "''")
}

impl MetadataChangeFromTo {
    pub fn sql_forward(&self) -> Option<String> {
        match (&self.from, &self.to) {
            (Some(from), Some(to)) => Some(format!(
                "UPDATE metadata SET value = '{}' WHERE name = '{}' AND value = '{}'",
                escape_sql_string(to),
                escape_sql_string(&self.name),
                escape_sql_string(from)
            )),
            (None, Some(to)) => Some(format!(
                "INSERT INTO metadata (name, value) VALUES ('{}', '{}');",
                escape_sql_string(&self.name),
                escape_sql_string(to)
            )),
            (Some(from), None) => Some(format!(
                "DELETE FROM metadata WHERE name = '{}' AND value = '{}';",
                escape_sql_string(&self.name),
                escape_sql_string(from)
            )),
            (None, None) => None,
        }
    }

    pub fn sql_reverse(&self) -> Option<String> {
        match (&self.from, &self.to) {
            (Some(from), Some(to)) => Some(format!(
                "UPDATE metadata SET value = '{}' WHERE name = '{}' AND value = '{}'",
                escape_sql_string(from),
                escape_sql_string(&self.name),
                escape_sql_string(to)
            )),
            (None, Some(to)) => Some(format!(
                "DELETE FROM metadata WHERE name = '{}' AND value = '{}';",
                escape_sql_string(&self.name),
                escape_sql_string(to)
            )),
            (Some(from), None) => Some(format!(
                "INSERT INTO metadata (name, value) VALUES ('{}', '{}');",
                escape_sql_string(&self.name),
                escape_sql_string(from)
            )),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MetadataChange {
    pub changes: Vec<MetadataChangeFromTo>,
    pub forward: Patch,
    pub reverse: Patch,
    pub data: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PragmaChange {
    pub pragma: String,
    pub forward: String,
    pub reverse: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum DbChange {
    Metadata(MetadataChange),
    Pragma(PragmaChange),
    Unknown(Value),
}

impl DbChangeset {
    pub(crate) fn apply_to_conn(
        &self,
        conn: &rusqlite::Connection,
    ) -> Result<(), rusqlite::Error> {
        for change in self.changes.iter() {
            match change {
                DbChange::Pragma(pragma_change) => {
                    conn.execute(&pragma_change.forward, [])?;
                }
                DbChange::Metadata(metadata_change) => {
                    for change in metadata_change.changes.iter() {
                        match (&change.from, &change.to) {
                            (Some(_from), Some(to)) => {
                                metadata_set(conn, &change.name, &to)?;
                            }
                            (None, Some(to)) => {
                                metadata_set(conn, &change.name, &to)?;
                            }
                            (Some(_from), None) => {
                                metadata_delete(conn, &change.name)?;
                            }
                            _ => {}
                        }
                    }
                }
                DbChange::Unknown(value) => {
                    warn!("Unknown DbChangeType: {:?}", value);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DbChangeset {
    pub timestamp: String,
    pub changes: Vec<DbChange>,
}

impl DbChangeset {
    #[must_use]
    pub fn from_vec(changes: Vec<DbChange>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            changes,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
    #[must_use]
    pub fn sql_forward_reverse(&self) -> (String, String) {
        let mut sql_forward_str = String::new();
        let mut sql_reverse_str = String::new();
        for change in self.changes.iter() {
            match change {
                DbChange::Pragma(pragma_change) => {
                    sql_forward_str.push_str(&pragma_change.forward);
                    sql_forward_str.push('\n');
                    sql_reverse_str.push_str(&pragma_change.reverse);
                    sql_reverse_str.push('\n');
                }
                DbChange::Metadata(metadata_change) => {
                    for change in metadata_change.changes.iter() {
                        if let Some(sql) = change.sql_forward() {
                            sql_forward_str.push_str(&sql);
                            sql_forward_str.push('\n');
                        }
                        if let Some(sql) = change.sql_reverse() {
                            sql_reverse_str.push_str(&sql);
                            sql_reverse_str.push('\n');
                        }
                    }
                }
                DbChange::Unknown(value) => {
                    warn!("Unknown DbChangeType: {:?}", value);
                }
            }
        }
        // Remove the trailing newline, if any
        if sql_forward_str.ends_with('\n') {
            sql_forward_str.pop();
        }
        if sql_reverse_str.ends_with('\n') {
            sql_reverse_str.pop();
        }
        (sql_forward_str, sql_reverse_str)
    }
}

impl From<MetadataChange> for DbChange {
    fn from(change: MetadataChange) -> Self {
        Self::Metadata(change)
    }
}

impl From<DbChange> for DbChangeset {
    fn from(change: DbChange) -> Self {
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
            }
        }
        Ok(())
    }
}
