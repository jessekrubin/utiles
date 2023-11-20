use std::collections::HashMap;

use crate::mbtiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};

pub fn metadata2duplicates(
    rows: Vec<MbtilesMetadataRow>,
) -> HashMap<String, Vec<MbtilesMetadataRow>> {
    rows.into_iter()
        .fold(HashMap::new(), |mut acc, row| {
            acc.entry(row.name.clone())
                .or_insert_with(Vec::new)
                .push(row);
            acc
        })
        .into_iter()
        .filter(|(_k, v)| v.len() > 1)
        .collect()
}
pub fn metadata2map(rows: &MbtilesMetadataRows) -> HashMap<String, String> {
    // let map: HashMap<String, String> =
    return HashMap::from_iter(
        rows.iter().map(|row| (row.name.clone(), row.value.clone())),
    );
    // map
}
