use std::path::Path;

use tracing::{debug, info};

use crate::cli::args::TouchArgs;
use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::is_valid_page_size;
use crate::sqlite::{Sqlike3Async, SqliteError};
use crate::timestamp::timestamp_string;
use crate::UtilesError;

fn check_page_size(page_size: i64) -> UtilesResult<i64> {
    if is_valid_page_size(page_size) {
        Ok(page_size)
    } else {
        let e = SqliteError::InvalidPageSize(page_size.to_string());
        Err(e.into())
    }
}

pub(crate) async fn touch_main(args: &TouchArgs) -> UtilesResult<()> {
    let filepath = &args.filepath;
    let page_size = check_page_size(args.page_size.unwrap_or(4096))?;
    debug!("touch: {}", filepath);
    let dbtype: MbtType = args.mbtype();
    let dbtype_str = dbtype.to_string();
    // check that filepath does not exist already
    let fpth = Path::new(filepath);
    // check that has an extension...
    if fpth.extension().is_none() {
        Err(UtilesError::NoFspathExtension(filepath.to_string()))
    } else {
        let stem_str = fpth.file_stem();
        if stem_str.is_none() {
            Err(UtilesError::NoFspathExtension(filepath.to_string()))
        } else {
            let filename_no_ext = stem_str;
            match filename_no_ext {
                Some(filename_no_ext) => {
                    let filename_no_ext = {
                        match filename_no_ext.to_str() {
                            Some(filename_no_ext) => filename_no_ext.to_string(),
                            None => {
                                return Err(UtilesError::PathConversionError(
                                    filepath.to_string(),
                                ));
                            }
                        }
                    };
                    let mbtiles =
                        MbtilesClientAsync::open_new(filepath, Some(dbtype)).await?;
                    info!("Created mbtiles: {:?}", filepath);
                    mbtiles.metadata_set("name", &filename_no_ext).await?;
                    mbtiles.metadata_set("mbtype", dbtype_str.as_str()).await?;
                    // current iso datetimestamp
                    let now_str = timestamp_string();
                    mbtiles.metadata_set("ctime", now_str.as_str()).await?;
                    mbtiles.metadata_set("mtime", now_str.as_str()).await?;
                    mbtiles.pragma_page_size_set(page_size).await?;
                    mbtiles.vacuum().await?;
                    Ok(())
                }
                None => Err(UtilesError::NoFspathStem(filepath.to_string())),
            }
        }
    }
}
