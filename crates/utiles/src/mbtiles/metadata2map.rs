use std::collections::HashMap;

use crate::mbtiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};

pub fn metadata2duplicates(
    rows: Vec<MbtilesMetadataRow>,
) -> HashMap<String, Vec<MbtilesMetadataRow>> {
    let mut map: HashMap<String, Vec<MbtilesMetadataRow>> = HashMap::new();
    for row in rows {
        map.entry(row.name.clone()).or_insert(Vec::new()).push(row);
    }

    // filter out the non-duplicates
    map.into_iter().filter(|(_k, v)| v.len() > 1).collect()
}

pub fn metadata2map(
    rows: MbtilesMetadataRows,
) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::from_iter(
        rows.iter().map(|row| (row.name.clone(), row.value.clone())),
    );
    map
}
