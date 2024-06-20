use std::collections::BTreeMap;

use serde_json::Value;

use crate::mbt::metadata_row::{MbtMetadataRow, MbtilesMetadataRows};

/// Return a `HashMap<String, Vec<MbtilesMetadataRow>>` of duplicate metadata rows
#[must_use]
pub fn metadata2duplicates(
    rows: Vec<MbtMetadataRow>,
) -> BTreeMap<String, Vec<MbtMetadataRow>> {
    rows.into_iter()
        .fold(
            BTreeMap::new(),
            |mut acc: BTreeMap<String, Vec<MbtMetadataRow>>, row| {
                acc.entry(row.name.clone()).or_default().push(row);
                acc
            },
        )
        .into_iter()
        .filter(|(_k, v)| v.len() > 1)
        .collect()
}
#[must_use]
pub fn metadata_vec_has_duplicates(rows: &[MbtMetadataRow]) -> bool {
    rows.iter()
        .fold(BTreeMap::new(), |mut acc: BTreeMap<String, usize>, row| {
            *acc.entry(row.name.clone()).or_default() += 1;
            acc
        })
        .into_iter()
        .any(|(_k, v)| v > 1)
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
