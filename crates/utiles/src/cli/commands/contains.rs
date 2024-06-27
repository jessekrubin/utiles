use std::path::Path;
use tracing::debug;

use crate::errors::UtilesResult;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use utiles_core::LngLat;

/// Check if a mbtiles file contains a lnglat
///
/// Added by [dan-costello](https://github.com/dan-costello)
pub async fn contains_main(filepath: &str, lnglat: LngLat) -> UtilesResult<()> {
    debug!("contains: {filepath}");
    // check that filepath exists and is file
    let filepath = Path::new(filepath);
    assert!(
        filepath.exists(),
        "File does not exist: {}",
        filepath.display()
    );
    assert!(
        filepath.is_file(),
        "Not a file: {filepath}",
        filepath = filepath.display()
    );
    let mbtiles = MbtilesAsyncSqliteClient::open_existing(filepath).await?;
    let bbox = mbtiles.bbox().await?;
    let contains = bbox.contains_lnglat(&lnglat);
    debug!("contains: {contains}");
    println!("{contains}");
    Ok(())
}
