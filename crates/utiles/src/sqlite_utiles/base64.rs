use base64::{engine::general_purpose, Engine as _};

use rusqlite::functions::FunctionFlags;
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use tracing::{error, trace};

/// Return xxh3-64 hash of string/blob as an integer (i64) value.
///
/// Sqlite stores integers as 8-byte signed integers.
pub fn add_function_base64_encode(db: &Connection) -> rusqlite::Result<()> {
    trace!("Adding base64_encode function");
    db.create_scalar_function(
        "base64_encode",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            if ctx.len() != 1 {
                error!("called with unexpected number of arguments");
                return Err(InvalidParameterCount(ctx.len(), 1));
            }
            let raw = ctx.get_raw(0);
            match raw {
                ValueRef::Blob(b) | ValueRef::Text(b) => {
                    let b64_encoded = general_purpose::STANDARD.encode(b);
                    Ok(rusqlite::types::Value::Text(b64_encoded))
                }
                v => {
                    error!("called with unexpected argument type");
                    Err(InvalidFunctionParameterType(0, v.data_type()))
                }
            }
        },
    )
}
