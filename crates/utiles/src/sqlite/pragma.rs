use crate::sqlite::errors::{SqliteError, SqliteResult};
use crate::sqlite::page_size::{is_valid_page_size, pragma_page_size_get};

use rusqlite::{Connection, Error as RusqliteError, Result as RusqliteResult};

pub fn pragma_page_count(conn: &Connection) -> SqliteResult<i64> {
    let mut stmt = conn.prepare("PRAGMA page_count")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.ok_or(RusqliteError::QueryReturnedNoRows)?;
    let count: i64 = row.get(0)?;
    Ok(count)
}

pub fn pragma_freelist_count(conn: &Connection) -> SqliteResult<i64> {
    let mut stmt = conn.prepare("PRAGMA freelist_count")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.ok_or(RusqliteError::QueryReturnedNoRows)?;
    let count: i64 = row.get(0)?;
    Ok(count)
}

pub fn pragma_page_size(
    conn: &Connection,
    page_size: Option<i64>,
) -> SqliteResult<i64> {
    if let Some(page_size) = page_size {
        pragma_page_size_set(conn, page_size)
    } else {
        pragma_page_size_get(conn)
    }
}

pub fn pragma_page_size_set(conn: &Connection, page_size: i64) -> SqliteResult<i64> {
    if is_valid_page_size(page_size) {
        // set page size
        let current_page_size = pragma_page_size_get(conn)?;
        if current_page_size == page_size {
            return Ok(page_size);
        }
        let stmt_str = format!("PRAGMA page_size = {page_size}");
        conn.execute(&stmt_str, [])?;
        Ok(page_size)
    } else {
        Err(SqliteError::InvalidPageSize(format!(
            "Invalid page size: {page_size}",
        )))
    }
}

#[derive(Debug)]
pub struct PragmaTableListRow {
    pub schema: String,
    pub name: String,
    pub type_: String,
    pub ncol: i64,
    pub wr: bool,
    pub strict: bool,
}

pub fn pragma_table_list(conn: &Connection) -> RusqliteResult<Vec<PragmaTableListRow>> {
    let mut stmt = conn.prepare("PRAGMA table_list")?;
    let mapped_rows = stmt.query_map([], |row| {
        let schema: String = row.get(0)?;
        let name: String = row.get(1)?;
        let type_: String = row.get(2)?;
        let ncol: i64 = row.get(3)?;
        let wr: bool = row.get(4)?;
        let strict: bool = row.get(5)?;
        Ok(PragmaTableListRow {
            schema,
            name,
            type_,
            ncol,
            wr,
            strict,
        })
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaTableListRow>>>()?;
    Ok(rows)
}

#[derive(Debug)]
pub struct PragmaTableInfoRow {
    pub cid: i64,
    pub name: String,
    pub type_: String,
    pub notnull: bool,
    pub dflt_value: Option<String>,
    pub pk: bool,
}

pub fn pragma_table_info(
    conn: &Connection,
    table: &str,
) -> RusqliteResult<Vec<PragmaTableInfoRow>> {
    let stmt_str = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&stmt_str)?;
    let mapped_rows = stmt.query_map([], |row| {
        let cid: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let type_: String = row.get(2)?;
        let notnull: bool = row.get(3)?;
        let dflt_value: Option<String> = row.get(4)?;
        let pk: bool = row.get(5)?;
        Ok(PragmaTableInfoRow {
            cid,
            name,
            type_,
            notnull,
            dflt_value,
            pk,
        })
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaTableInfoRow>>>()?;
    Ok(rows)
}

#[derive(Debug)]
pub struct PragmaTableXInfoRow {
    pub cid: i64,
    pub name: String,
    pub type_: String,
    pub notnull: bool,
    pub dflt_value: Option<String>,
    pub pk: bool,
    pub hidden: bool,
}

pub fn pragma_table_xinfo(
    conn: &Connection,
    table: &str,
) -> RusqliteResult<Vec<PragmaTableXInfoRow>> {
    let stmt_str = format!("PRAGMA table_xinfo({table})");
    let mut stmt = conn.prepare(&stmt_str)?;
    let mapped_rows = stmt.query_map([], |row| {
        let cid: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let type_: String = row.get(2)?;
        let notnull: bool = row.get(3)?;
        let dflt_value: Option<String> = row.get(4)?;
        let pk: bool = row.get(5)?;
        let hidden: bool = row.get(6)?;
        Ok(PragmaTableXInfoRow {
            cid,
            name,
            type_,
            notnull,
            dflt_value,
            pk,
            hidden,
        })
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaTableXInfoRow>>>()?;
    Ok(rows)
}

/// Row returned by `PRAGMA database_list;`
#[derive(Debug)]
pub struct PragmaDatabaseListRow {
    pub seq: i64,
    pub name: String,
    pub file: String,
}

pub fn pragma_database_list(
    conn: &Connection,
) -> RusqliteResult<Vec<PragmaDatabaseListRow>> {
    let mut stmt = conn.prepare("PRAGMA database_list")?;
    let mapped_rows = stmt.query_map([], |row| {
        let seq: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let file: String = row.get(2)?;
        Ok(PragmaDatabaseListRow { seq, name, file })
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaDatabaseListRow>>>()?;
    Ok(rows)
}

#[derive(Debug)]
pub struct PragmaIndexListRow {
    pub seq: i64,
    pub name: String,
    pub unique: bool,
    pub origin: String,
    pub partial: bool,
}

pub fn pragma_index_list(
    conn: &Connection,
    table: &str,
) -> RusqliteResult<Vec<PragmaIndexListRow>> {
    let stmt_str = format!("PRAGMA index_list({table})");
    let mut stmt = conn.prepare(&stmt_str)?;

    let mapped_rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let row = PragmaIndexListRow {
            seq: row.get(0)?,
            name,
            unique: row.get(2)?,
            origin: row.get(3)?,
            partial: row.get(4)?,
        };
        Ok(row)
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaIndexListRow>>>()?;
    Ok(rows)
}

#[derive(Debug)]
pub struct PragmaIndexInfoRow {
    pub seqno: i64,
    pub cid: i64,
    pub name: String,
}

pub fn pragma_index_info(
    conn: &Connection,
    index: &str,
) -> RusqliteResult<Vec<PragmaIndexInfoRow>> {
    let stmt_str = format!("PRAGMA index_info({index})");
    let mut stmt = conn.prepare(&stmt_str)?;
    let mapped_rows = stmt.query_map([], |row| {
        let row = PragmaIndexInfoRow {
            seqno: row.get(0)?,
            cid: row.get(1)?,
            name: row.get(2)?,
        };
        Ok(row)
    })?;
    let rows = mapped_rows.collect::<RusqliteResult<Vec<PragmaIndexInfoRow>>>()?;
    Ok(rows)
}
