use std::path::Path;
use tracing::{debug, error};

use crate::utilesqlite::Mbtiles;
use utiles_core::LngLat;

/// Check if an mbtiles file contains a lnglat
///
/// Added by [dan-costello](https://github.com/dan-costello)
pub fn contains_main(filepath: &str, lnglat: LngLat) {
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
    let mbtiles: Mbtiles = Mbtiles::from(filepath);
    let contains = mbtiles.contains(lnglat);
    if contains.is_err() {
        error!("contains error: {:?}", contains);
        println!("contains error: {contains:?}");
    } else {
        println!("{}", contains.unwrap());
    }
}
