use crate::UtilesError;
use rusqlite::Connection;

use crate::mbt::MbtType;
use crate::sqlite::RusqliteResult;
use crate::UtilesResult;

const IS_FLAT_MBTILES_QUERY: &str = include_str!("sql/is-flat-mbtiles-query.sql");
const IS_NORM_MBTILES_QUERY: &str = include_str!("sql/is-norm-mbtiles-query.sql");
const IS_HASH_MBTILES_QUERY: &str = include_str!("sql/is-hash-mbtiles-query.sql");
const IS_TIPPECANOE_MBTILES_QUERY: &str =
    include_str!("sql/is-tippecanoe-mbtiles-query.sql");

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

pub fn is_tippecanoe_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_TIPPECANOE_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn query_mbtiles_type(conn: &Connection) -> UtilesResult<MbtType> {
    let is_tippecanoe =
        is_tippecanoe_mbtiles(conn).map_err(UtilesError::RusqliteError)?;
    if is_tippecanoe {
        return Ok(MbtType::Tippecanoe);
    }
    let is_norm = is_norm_mbtiles(conn).map_err(UtilesError::RusqliteError)?;
    if is_norm {
        return Ok(MbtType::Norm);
    }
    let is_hash = is_tiles_with_hash(conn).map_err(UtilesError::RusqliteError)?;

    if is_hash {
        return Ok(MbtType::Hash);
    }
    let is_flat = is_flat_mbtiles(conn).map_err(UtilesError::RusqliteError)?;
    Ok(if is_flat {
        MbtType::Flat
    } else {
        MbtType::Unknown
    })
}

pub fn default_mbtiles_settings(conn: &Connection) -> UtilesResult<()> {
    // page size 512
    {
        conn.execute_batch("PRAGMA page_size = 512;")?;
    }
    // encoding UTF-8
    {
        conn.execute_batch("PRAGMA encoding = 'UTF-8';")?;
    }
    Ok(())
}
