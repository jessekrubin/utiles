use std::path::Path;

use tracing::{debug, info};

use crate::cli::args::TouchArgs;
use crate::errors::UtilesResult;
use crate::sqlite::SqliteError;
use crate::sqlite::{is_valid_page_size, Sqlike3};
use crate::utilesqlite::Mbtiles;

fn check_page_size(page_size: i64) -> UtilesResult<i64> {
    if is_valid_page_size(page_size) {
        Ok(page_size)
    } else {
        let e = SqliteError::InvalidPageSize(page_size.to_string());
        Err(e.into())
    }
}

pub fn touch_main(args: &TouchArgs) -> UtilesResult<()> {
    let filepath = &args.filepath;

    let page_size = check_page_size(args.page_size.unwrap_or(512))?;
    debug!("touch: {}", filepath);
    // check that filepath does not exist already
    let fpth = Path::new(filepath);
    assert!(!fpth.exists(), "Already exists: {}", fpth.display());
    let mbtiles = Mbtiles::create(filepath, None)?;
    info!("Created mbtiles: {:?}", filepath);
    let filename_no_ext = fpth.file_stem().unwrap().to_str().unwrap();
    mbtiles.metadata_set("name", filename_no_ext).unwrap();
    mbtiles.pragma_page_size_set(page_size)?;
    mbtiles.vacuum()?;
    Ok(())
}
