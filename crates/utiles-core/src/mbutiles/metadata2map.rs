use std::collections::HashMap;

use crate::mbutiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};

/// Return a `HashMap<String, Vec<MbtilesMetadataRow>>` of duplicate metadata rows
#[must_use]
pub fn metadata2duplicates(
    rows: Vec<MbtilesMetadataRow>,
) -> HashMap<String, Vec<MbtilesMetadataRow>> {
    rows.into_iter()
        .fold(
            HashMap::new(),
            |mut acc: std::collections::HashMap<
                std::string::String,
                Vec<MbtilesMetadataRow>,
            >,
             row| {
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
pub fn metadata2map(rows: &MbtilesMetadataRows) -> HashMap<String, String> {
    // let map: HashMap<String, String> =
    return HashMap::from_iter(
        rows.iter().map(|row| (row.name.clone(), row.value.clone())),
    );
    // map
}
