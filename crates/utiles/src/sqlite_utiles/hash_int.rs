use rusqlite::functions::FunctionFlags;
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use tracing::{debug, error};
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

/// Return xxh3 hash of string/blob as an integer (i64) value.
///
/// Sqlite stores integers as 8-byte signed integers.
pub fn add_function_xxh3_int(db: &Connection) -> rusqlite::Result<()> {
    debug!("Adding xxh3_int function");
    db.create_scalar_function(
        "xxh3_i64",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            if ctx.len() != 1 {
                error!("called with unexpected number of arguments");
                return Err(InvalidParameterCount(ctx.len(), 1));
            }
            let raw = ctx.get_raw(0);
            match raw {
                ValueRef::Text(s) => {
                    let a = const_xxh3(s);
                    Ok(rusqlite::types::Value::Integer(a as i64))
                }
                ValueRef::Blob(b) => {
                    let a = const_xxh3(b);
                    Ok(rusqlite::types::Value::Integer(a as i64))
                }
                v => {
                    error!("called with unexpected argument type");
                    Err(InvalidFunctionParameterType(0, v.data_type()))
                }
            }
        },
    )
}
