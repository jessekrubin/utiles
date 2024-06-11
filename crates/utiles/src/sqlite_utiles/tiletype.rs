use rusqlite::functions::FunctionFlags;
use rusqlite::Connection;
use tracing::debug;
use utiles_core::tile_type::tiletype_str;

/// Scalar function to return tile-type string.
pub fn add_function_ut_tiletype(db: &Connection) -> rusqlite::Result<()> {
    debug!("Adding ut_tiletype function");
    db.create_scalar_function(
        "ut_tiletype",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");
            // assert arg is blob
            let blob = ctx.get_raw(0).as_blob()?;
            let tt = tiletype_str(blob);
            Ok(tt)
        },
    )
}
