use std::path::Path;

use tokio::fs::read_to_string;

use crate::errors::UtilesResult;
use crate::mbt::metadata_row::MbtilesMetadataJson;

// read metadata json from filepath...
pub async fn read_metadata_json(
    filepath: impl AsRef<Path>,
) -> UtilesResult<MbtilesMetadataJson> {
    let fpath = filepath.as_ref().to_owned();
    let data = read_to_string(fpath).await?;
    let v = serde_json::from_str::<MbtilesMetadataJson>(&data)?;
    Ok(v)
}
