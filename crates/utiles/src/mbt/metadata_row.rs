use crate::errors::UtilesResult;
use json_patch::Patch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
        MbtMetadataRow { name, value }
    }
}

pub type MbtilesMetadataRows = Vec<MbtMetadataRow>;

impl From<&MbtMetadataRow> for MbtilesMetadataRowParsed {
    fn from(row: &MbtMetadataRow) -> Self {
        let value = match row.value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(row.value.clone()),
        };
        MbtilesMetadataRowParsed {
            name: row.name.to_string(),
            value,
        }
    }
}

impl MbtilesMetadataJson {
    #[must_use]
    pub fn from_key_value(key: &str, value: &str) -> MbtilesMetadataJson {
        let mut obj = BTreeMap::new();
        // try and parse the value...
        let val = match value.parse::<Value>() {
            Ok(v) => v,
            Err(_) => Value::String(value.to_string()),
        };
        obj.insert(key.to_string(), val);
        MbtilesMetadataJson::Obj(obj)
    }

    pub fn delete(&mut self, key: &str) {
        match self {
            MbtilesMetadataJson::Obj(obj) => {
                obj.remove(key);
            }
            MbtilesMetadataJson::Arr(arr) => {
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
            MbtilesMetadataJson::Obj(obj) => {
                obj.insert(key.to_string(), val);
            }
            MbtilesMetadataJson::Arr(arr) => {
                arr.push(MbtilesMetadataRowParsed {
                    name: key.to_string(),
                    value: val,
                });
            }
        }
    }

    #[must_use]
    pub fn as_obj(&self) -> BTreeMap<String, Value> {
        match self {
            MbtilesMetadataJson::Obj(obj) => obj.clone(),
            MbtilesMetadataJson::Arr(arr) => {
                let obj: BTreeMap<String, Value> =
                    arr.iter().fold(BTreeMap::new(), |mut acc, row| {
                        acc.insert(row.name.clone(), row.value.clone());
                        acc
                    });
                obj
            }
        }
    }

    #[must_use]
    pub fn as_arr(&self) -> Vec<MbtilesMetadataRowParsed> {
        match self {
            MbtilesMetadataJson::Arr(arr) => arr.clone(),
            MbtilesMetadataJson::Obj(obj) => {
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
            MbtilesMetadataJson::Obj(obj) => {
                if pretty {
                    serde_json::to_string_pretty(obj)
                } else {
                    serde_json::to_string(obj)
                }
            }
            MbtilesMetadataJson::Arr(arr) => {
                if pretty {
                    serde_json::to_string_pretty(arr)
                } else {
                    serde_json::to_string(arr)
                }
            }
        }
    }

    pub fn diff(
        &self,
        other: &MbtilesMetadataJson,
        merge: bool,
    ) -> UtilesResult<(Patch, Patch, Value)> {
        let self_value = serde_json::to_value(self)?;
        let mut merged = self_value.clone();
        let other_value = serde_json::to_value(other)?;
        if merge {
            json_patch::merge(&mut merged, &other_value);
        }
        let forward_patch = json_patch::diff(&self_value, &merged);
        let reverse_patch = json_patch::diff(&merged, &self_value);
        let mut patched_data = self_value.clone();
        json_patch::patch(&mut patched_data, &forward_patch)?;
        Ok((forward_patch, reverse_patch, patched_data))
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
            MbtilesMetadataJsonRaw::Arr(rows.clone())
        } else {
            let obj: BTreeMap<String, String> =
                rows.iter().fold(BTreeMap::new(), |mut acc, row| {
                    acc.insert(row.name.clone(), row.value.clone());
                    acc
                });
            MbtilesMetadataJsonRaw::Obj(obj)
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
            MbtilesMetadataJson::Arr(arr)
        } else {
            let obj: BTreeMap<String, Value> =
                arr.iter().fold(BTreeMap::new(), |mut acc, row| {
                    acc.insert(row.name.clone(), row.value.clone());
                    acc
                });
            MbtilesMetadataJson::Obj(obj)
        }
    }
}
