use std::path::Path;

use crate::cli::args::TouchArgs;
use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::sqlite::SqliteError;
use crate::sqlite::{is_valid_page_size, Sqlike3};
use crate::utilesqlite::Mbtiles;
use crate::UtilesError;
use chrono;
use tracing::{debug, info};

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
    let dbtype: MbtType = args.mbtype();
    let dbtype_str = dbtype.to_string();
    // check that filepath does not exist already
    let fpth = Path::new(filepath);
    // check that has an extension...
    if fpth.extension().is_none() {
        Err(UtilesError::NoFspathExtension(filepath.to_string()))
    } else {
        let filename_no_ext = fpth.file_stem().unwrap().to_str().unwrap();

        assert!(!fpth.exists(), "Already exists: {}", fpth.display());
        let mbtiles = Mbtiles::create(filepath, Some(dbtype))?;
        info!("Created mbtiles: {:?}", filepath);
        mbtiles.metadata_set("name", filename_no_ext)?;
        mbtiles.metadata_set("mbtype", dbtype_str.to_string().as_str())?;
        // current iso datetimestamp
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();
        mbtiles.metadata_set("ctime", now_str.as_str())?;
        mbtiles.metadata_set("mtime", now_str.as_str())?;
        mbtiles.pragma_page_size_set(page_size)?;
        mbtiles.vacuum()?;
        Ok(())
    }
}
