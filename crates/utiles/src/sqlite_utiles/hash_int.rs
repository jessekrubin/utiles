use std::hash::Hasher;

use fnv;
use fnv::FnvHasher;
use rusqlite::Connection;
use rusqlite::Error::{InvalidFunctionParameterType, InvalidParameterCount};
use rusqlite::functions::FunctionFlags;
use rusqlite::types::ValueRef;
use tracing::{error, trace};
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;
use xxhash_rust::const_xxh64::xxh64 as const_xxh64;

/// Return xxh3-64 hash of string/blob as an integer (i64) value.
///
/// Sqlite stores integers as 8-byte signed integers.
pub(super) fn add_function_xxh3_i64(db: &Connection) -> rusqlite::Result<()> {
    trace!("Adding xxh3_i64 function");
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
                ValueRef::Blob(b) | ValueRef::Text(b) => {
                    let hash_bytes = const_xxh3(b);
                    // convert to i64 big-endian
                    let has_int = i64::from_be_bytes(hash_bytes.to_be_bytes());
                    Ok(rusqlite::types::Value::Integer(has_int))
                }
                v => {
                    error!("called with unexpected argument type");
                    Err(InvalidFunctionParameterType(0, v.data_type()))
                }
            }
        },
    )
}

/// Return xxh32 hash of string/blob as integer (i64) value.
///
pub(super) fn add_function_xxh64_i64(db: &Connection) -> rusqlite::Result<()> {
    trace!("Adding xxh64_i64 function");
    db.create_scalar_function(
        "xxh64_i64",
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
                    let hash_bytes = const_xxh64(b, 0);
                    let as_int = i64::from_be_bytes(hash_bytes.to_be_bytes());
                    Ok(rusqlite::types::Value::Integer(as_int))
                }
                v => {
                    error!("called with unexpected argument type");
                    Err(InvalidFunctionParameterType(0, v.data_type()))
                }
            }
        },
    )
}

#[inline]
fn fnv1a_u64(bytes: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();
    hasher.write(bytes);
    hasher.finish()
}

pub(super) fn add_function_fnv_i64(db: &Connection) -> rusqlite::Result<()> {
    trace!("Adding fnv_i64 function");
    db.create_scalar_function(
        "fnv_i64",
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
                    let hash_i64_be = i64::from_be_bytes(fnv1a_u64(b).to_be_bytes());
                    Ok(rusqlite::types::Value::Integer(hash_i64_be))
                }
                v => {
                    error!("called with unexpected argument type");
                    Err(InvalidFunctionParameterType(0, v.data_type()))
                }
            }
        },
    )
}
