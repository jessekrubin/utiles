use rusqlite::functions::FunctionFlags;
use rusqlite::Connection;
use tracing::{debug, error};

/// Scalar function to return the size of a tile if it is a square image.
///
/// Returns -1 if the image is not square.
///
/// Returns NULL if not an image or errors... could do more here
pub fn add_function_ut_tilesize(db: &Connection) -> rusqlite::Result<()> {
    debug!("Adding ut_tilesize function");
    db.create_scalar_function(
        "ut_tilesize",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");
            let blob = ctx.get_raw(0).as_blob()?;

            let size: Option<i64> = match imagesize::blob_size(blob) {
                Ok(imgsize) => {
                    if imgsize.width == imgsize.height {
                        Some(imgsize.width as i64)
                    } else {
                        // -1 is there is a problem and img is not square
                        Some(-1)
                    }
                }
                Err(e) => {
                    error!("error getting image size: {}", e);
                    None
                }
            };
            let v = match size {
                Some(s) => rusqlite::types::Value::Integer(s),
                None => rusqlite::types::Value::Null,
            };
            Ok(v)
        },
    )
}
