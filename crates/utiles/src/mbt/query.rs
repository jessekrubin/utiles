use rusqlite::Connection;

use crate::mbt::MbtType;
use crate::sqlite::RusqliteResult;
use crate::UtilesResult;

const IS_FLAT_MBTILES_QUERY: &str = include_str!("sql/is_flat_mbtiles_query.sql");
const IS_NORM_MBTILES_QUERY: &str = include_str!("sql/is_norm_mbtiles_query.sql");
const IS_HASH_MBTILES_QUERY: &str = include_str!("sql/is_hash_mbtiles_query.sql");

pub fn is_tiles_with_hash(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_HASH_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_flat_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_FLAT_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_norm_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_NORM_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn query_mbtiles_type(conn: &Connection) -> UtilesResult<MbtType> {
    let is_norm = is_norm_mbtiles(conn)?;
    if is_norm {
        return Ok(MbtType::Norm);
    }
    let is_hash = is_tiles_with_hash(conn)?;
    if is_hash {
        return Ok(MbtType::Hash);
    }
    let is_flat = is_flat_mbtiles(conn)?;
    Ok(if is_flat {
        MbtType::Flat
    } else {
        MbtType::Unknown
    })
}
