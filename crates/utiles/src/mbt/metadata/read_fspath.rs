use crate::mbt::metadata_row::MbtilesMetadataJson;
use crate::UtilesError;
use crate::UtilesResult;
use std::path::Path;
use tokio::fs::read_to_string;

// read metadata json from filepath...
pub async fn read_metadata_json(
    filepath: impl AsRef<Path>,
) -> UtilesResult<MbtilesMetadataJson> {
    let filepath = filepath.as_ref().to_str();
    match filepath {
        Some(filepath) => {
            let data = read_to_string(filepath).await?;
            let v = serde_json::from_str::<MbtilesMetadataJson>(&data)?;
            Ok(v)
        }
        None => Err(UtilesError::PathConversionError(
            filepath.unwrap_or("UNKNOWN").to_string(),
        )),
    }
}
