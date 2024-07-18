use rusqlite::{
    Connection, DatabaseName, Error as RusqliteError, Result as RusqliteResult,
};
use tracing::debug;

pub fn journal_mode(conn: &Connection) -> RusqliteResult<String> {
    let jm = conn.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
    Ok(jm)
}

pub fn journal_mode_set(
    conn: &Connection,
    mode: &str,
    schema_name: Option<DatabaseName>,
) -> RusqliteResult<bool> {
    let current_mode = conn.pragma_query_value(schema_name, "journal_mode", |row| {
        let val: String = row.get(0)?;
        Ok(val)
    })?;

    if current_mode == mode {
        debug!("journal_mode_set: current mode == mode: {}", mode);
        Ok(false)
    } else {
        debug!(
            "journal_mode_set: current mode != mode: {} != {}",
            current_mode, mode
        );
        conn.pragma_update(schema_name, "journal_mode", mode)?;
        Ok(true)
    }
}
pub fn pragma_page_count(conn: &Connection) -> RusqliteResult<i64> {
    let mut stmt = conn.prepare("PRAGMA page_count")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.ok_or(RusqliteError::QueryReturnedNoRows)?;
    let count: i64 = row.get(0)?;
    Ok(count)
}

pub fn application_id(conn: &Connection) -> RusqliteResult<u32> {
    let app_id = conn.pragma_query_value(None, "application_id", |row| row.get(0))?;
    Ok(app_id)
}

pub fn application_id_set(conn: &Connection, app_id: u32) -> RusqliteResult<u32> {
    let current_app_id = application_id(conn)?;
    if current_app_id == app_id {
        debug!("application_id_set: current app_id == app_id: {}", app_id);
        Ok(current_app_id)
    } else {
        debug!(
            "application_id_set: current app_id != app_id: {} != {}",
            current_app_id, app_id
        );
        conn.pragma_update(None, "application_id", app_id)?;
        Ok(app_id)
    }
}

pub fn magic_number(conn: &Connection) -> RusqliteResult<u32> {
    application_id(conn)
}

pub fn pragma_freelist_count(conn: &Connection) -> RusqliteResult<i64> {
    let freelist_count = conn.pragma_query_value(None, "freelist_count", |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(freelist_count)
}

pub fn pragma_page_size_get(conn: &Connection) -> RusqliteResult<i64> {
    let r = conn.pragma_query_value(None, "page_size", |row| row.get(0))?;
    Ok(r)
}

pub fn pragma_page_size_set(conn: &Connection, page_size: i64) -> RusqliteResult<i64> {
    // set page size
    let current_page_size = pragma_page_size_get(conn)?;
    if current_page_size == page_size {
        Ok(page_size)
    } else {
        conn.pragma_update(None, "page_size", page_size)?;
        Ok(page_size)
    }
}

pub fn pragma_page_size(
    conn: &Connection,
    page_size: Option<i64>,
) -> RusqliteResult<i64> {
    if let Some(page_size) = page_size {
        pragma_page_size_set(conn, page_size)
    } else {
        pragma_page_size_get(conn)
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use crate::sqlite::open;

    use super::*;

    #[test]
    fn journal_mode_pragma() {
        let conn = open(":memory:").unwrap();
        let jm = journal_mode(&conn).unwrap();
        assert_eq!(jm, "memory");
        let jm_set_res = journal_mode_set(&conn, "wal", None).unwrap();
        assert!(jm_set_res);
    }

    #[test]
    fn pragma_page_size_pragma() {
        let conn = open(":memory:").unwrap();
        let page_size = pragma_page_size(&conn, None).unwrap();
        assert_eq!(page_size, 4096);
        let page_size_set_res = pragma_page_size_set(&conn, 8192).unwrap();
        assert_eq!(page_size_set_res, 8192);
    }

    #[test]
    fn pragma_application_id_pragma() {
        let conn = open(":memory:").unwrap();
        let app_id = application_id(&conn).unwrap();
        assert_eq!(app_id, 0);
        let app_id_set_res = application_id_set(&conn, 1).unwrap();
        assert_eq!(app_id_set_res, 1);
    }
}
