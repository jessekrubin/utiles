use std::collections::BTreeMap;

use serde_json::Value;

use crate::mbutiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};

/// Return a `HashMap<String, Vec<MbtilesMetadataRow>>` of duplicate metadata rows
#[must_use]
pub fn metadata2duplicates(
    rows: Vec<MbtilesMetadataRow>,
) -> BTreeMap<String, Vec<MbtilesMetadataRow>> {
    rows.into_iter()
        .fold(
            BTreeMap::new(),
            |mut acc: BTreeMap<String, Vec<MbtilesMetadataRow>>, row| {
                acc.entry(row.name.clone()).or_default().push(row);
                acc
            },
        )
        .into_iter()
        .filter(|(_k, v)| v.len() > 1)
        .collect()
}

/// Convert a `MbtilesMetadataRows` to a `HashMap<String, String>`
#[must_use]
pub fn metadata2map(rows: &MbtilesMetadataRows) -> BTreeMap<String, String> {
    // return HashMap::from_iter(
    //     // rows.iter().map(|row| (row.name.clone(), row.value.clone())),
    // );
    rows.iter()
        .map(|row| (row.name.clone(), row.value.clone()))
        .collect::<BTreeMap<_, _>>()
}

/// Convert `MbtilesMetadataRows` to a `HashMap<String, Value>`
/// where `Value` is a `serde_json::Value`
#[must_use]
pub fn metadata2map_val(
    rows: &MbtilesMetadataRows,
) -> BTreeMap<String, serde_json::Value> {
    rows.iter()
        .map(|row| {
            let v = match row.value.parse::<Value>() {
                Ok(v) => v,
                Err(_) => Value::String(row.value.clone()),
            };
            (row.name.clone(), v)
        })
        .collect()
}
