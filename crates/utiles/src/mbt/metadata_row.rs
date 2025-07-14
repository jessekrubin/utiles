use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::mbt::{MetadataChange, MetadataChangeFromTo};

/// Metadata row struct for `mbtiles` metadata table
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MbtilesMetadataJsonRaw {
    Obj(BTreeMap<String, String>),
    Arr(Vec<MbtMetadataRow>),
}

/// Metadata Json enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MbtilesMetadataJson {
    Obj(BTreeMap<String, Value>),
    Arr(Vec<MbtilesMetadataRowParsed>),
    // ObjRaw(BTreeMap<String, String>),
    // ArrRaw(Vec<MbtMetadataRow>),
}

impl MbtMetadataRow {
    /// Create a new `MbtilesMetadataRow`
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtMetadataRow>;

impl From<&MbtMetadataRow> for MbtilesMetadataRowParsed {
    fn from(row: &MbtMetadataRow) -> Self {
        let value = match row.value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(row.value.clone()),
        };
        Self {
            name: row.name.to_string(),
            value,
        }
    }
}

impl MbtilesMetadataJson {
    #[must_use]
    pub fn from_key_value(key: &str, value: &str) -> Self {
        let mut obj = BTreeMap::new();
        // try and parse the value...
        let val = match value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(value.to_string()),
        };
        obj.insert(key.to_string(), val);
        Self::Obj(obj)
    }

    pub fn delete(&mut self, key: &str) {
        match self {
            Self::Obj(obj) => {
                obj.remove(key);
            }
            Self::Arr(arr) => {
                arr.retain(|row| row.name != key);
            }
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        let val = match value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(value.to_string()),
        };
        match self {
            Self::Obj(obj) => {
                obj.insert(key.to_string(), val);
            }
            Self::Arr(arr) => {
                arr.push(MbtilesMetadataRowParsed {
                    name: key.to_string(),
                    value: val,
                });
            }
        }
    }

    pub fn update(&mut self, key: &str, value: &str) {
        let val = match value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(value.to_string()),
        };
        match self {
            Self::Obj(obj) => {
                // if key exists, and value is different update value
                if let Some(v) = obj.get_mut(key) {
                    if v != &val {
                        *v = val;
                    }
                } else {
                    // else insert new key value pair
                    obj.insert(key.to_string(), val);
                } // baboom
            }
            Self::Arr(arr) => {
                // if key exists update value
                if let Some(row) = arr.iter_mut().find(|row| row.name == key) {
                    if row.value != val {
                        row.value = val;
                    }
                } else {
                    // else insert new row
                    arr.push(MbtilesMetadataRowParsed {
                        name: key.to_string(),
                        value: val,
                    });
                }
            }
        }
    }
    #[must_use]
    pub fn as_obj(&self) -> BTreeMap<String, Value> {
        match self {
            Self::Obj(obj) => obj.clone(),
            Self::Arr(arr) => {
                let obj: BTreeMap<String, Value> =
                    arr.iter().fold(BTreeMap::new(), |mut acc, row| {
                        acc.insert(row.name.clone(), row.value.clone());
                        acc
                    });
                obj
            }
        }
    }
    /// Returns the raw object representation of the metadata
    /// where the value is json-stringified if it is not a string
    #[must_use]
    pub fn as_obj_raw(&self) -> BTreeMap<String, String> {
        match self {
            Self::Obj(obj) => {
                let obj: BTreeMap<String, String> = obj
                    .iter()
                    .map(|(k, v)| {
                        let val = match v {
                            Value::String(s) => s.clone(),
                            _ => serde_json::to_string(v).unwrap_or_default(),
                        };
                        (k.clone(), val)
                    })
                    .collect();
                obj
            }
            Self::Arr(arr) => {
                let obj: BTreeMap<String, String> = arr
                    .iter()
                    .map(|row| {
                        let val = match &row.value {
                            Value::String(s) => s.clone(),
                            _ => serde_json::to_string(&row.value).unwrap_or_default(),
                        };
                        (row.name.clone(), val)
                    })
                    .collect();
                obj
            }
        }
    }

    #[must_use]
    pub fn as_arr(&self) -> Vec<MbtilesMetadataRowParsed> {
        match self {
            Self::Arr(arr) => arr.clone(),
            Self::Obj(obj) => {
                let arr: Vec<MbtilesMetadataRowParsed> = obj
                    .iter()
                    .map(|(k, v)| MbtilesMetadataRowParsed {
                        name: k.clone(),
                        value: v.clone(),
                    })
                    .collect();
                arr
            }
        }
    }

    pub fn stringify(&self, pretty: bool) -> Result<String, serde_json::Error> {
        match self {
            Self::Obj(obj) => {
                if pretty {
                    serde_json::to_string_pretty(obj)
                } else {
                    serde_json::to_string(obj)
                }
            }
            Self::Arr(arr) => {
                if pretty {
                    serde_json::to_string_pretty(arr)
                } else {
                    serde_json::to_string(arr)
                }
            }
        }
    }

    pub fn diff_changes(
        &self,
        other: &Value,
    ) -> UtilesResult<Vec<MetadataChangeFromTo>> {
        let from_map = self.as_obj_raw();
        let to_map: BTreeMap<String, String> = match other {
            Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        Value::String(s) => s.clone(),
                        _ => serde_json::to_string(v).unwrap_or_default(),
                    };
                    (k.clone(), val)
                })
                .collect(),
            _ => {
                return Err(UtilesError::MetadataError(
                    "Value is not an object".to_string(),
                ));
            }
        };
        let all_keys = from_map.keys().chain(to_map.keys());
        let changes = all_keys
            .filter_map(|k| {
                let from = from_map.get(k);
                let to = to_map.get(k);
                if from == to {
                    None
                } else {
                    Some(MetadataChangeFromTo {
                        name: k.clone(),
                        from: from.cloned(),
                        to: to.cloned(),
                    })
                }
            })
            .collect();
        Ok(changes)
    }

    pub fn diff(&self, other: &Self, merge: bool) -> UtilesResult<MetadataChange> {
        let self_value = serde_json::to_value(self)?;
        let mut merged = self_value.clone();
        let other_value = serde_json::to_value(other)?;

        let self_value_json_string = serde_json::to_string_pretty(&self_value)?;
        let other_value_json_string = serde_json::to_string_pretty(&other_value)?;
        debug!("self_value: {}", self_value_json_string);
        debug!("other_value: {}", other_value_json_string);
        if merge {
            debug!("merging...");
            json_patch::merge(&mut merged, &other_value);
        }
        let forward_patch = json_patch::diff(&self_value, &merged);
        let reverse_patch = json_patch::diff(&merged, &self_value);
        let mut patched_data = self_value;
        json_patch::patch(&mut patched_data, &forward_patch)?;
        let changes = self.diff_changes(&patched_data)?;
        debug!("calculated changes: {:?}", changes);
        Ok(MetadataChange {
            forward: forward_patch,
            reverse: reverse_patch,
            data: patched_data,
            changes,
        })
    }
}

impl From<&Vec<MbtMetadataRow>> for MbtilesMetadataJsonRaw {
    fn from(rows: &Vec<MbtMetadataRow>) -> Self {
        let has_duplicates = rows
            .iter()
            .fold(BTreeMap::new(), |mut acc: BTreeMap<String, usize>, row| {
                *acc.entry(row.name.clone()).or_default() += 1;
                acc
            })
            .into_iter()
            .any(|(_k, v)| v > 1);

        if has_duplicates {
            Self::Arr(rows.clone())
        } else {
            let obj: BTreeMap<String, String> =
                rows.iter().fold(BTreeMap::new(), |mut acc, row| {
                    acc.insert(row.name.clone(), row.value.clone());
                    acc
                });
            Self::Obj(obj)
        }
    }
}
impl From<&Vec<MbtMetadataRow>> for MbtilesMetadataJson {
    fn from(rows: &Vec<MbtMetadataRow>) -> Self {
        let arr: Vec<MbtilesMetadataRowParsed> =
            rows.iter().map(MbtilesMetadataRowParsed::from).collect();
        let has_duplicates = arr
            .iter()
            .fold(BTreeMap::new(), |mut acc: BTreeMap<String, usize>, row| {
                *acc.entry(row.name.clone()).or_default() += 1;
                acc
            })
            .into_iter()
            .any(|(_k, v)| v > 1);

        if has_duplicates {
            Self::Arr(arr)
        } else {
            let obj: BTreeMap<String, Value> =
                arr.iter().fold(BTreeMap::new(), |mut acc, row| {
                    acc.insert(row.name.clone(), row.value.clone());
                    acc
                });
            Self::Obj(obj)
        }
    }
}
