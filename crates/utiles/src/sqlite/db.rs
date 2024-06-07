use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;

use crate::errors::UtilesResult;
use crate::sqlite::{
    pragma_database_list, pragma_index_list, PragmaIndexListRow, RusqliteResult,
    Sqlike3,
};
use crate::UtilesError;

pub struct SqliteDb {
    pub conn: Connection,
}

impl Sqlike3 for SqliteDb {
    fn conn(&self) -> &Connection {
        &self.conn
    }
}

impl SqliteDb {
    pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let conn = open_existing(path)?;
        Ok(SqliteDb { conn })
    }
}

impl From<Connection> for SqliteDb {
    fn from(conn: Connection) -> Self {
        SqliteDb { conn }
    }
}

pub fn open(path: &str) -> RusqliteResult<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Connection> {
    let filepath = path.as_ref();

    let fspath = filepath
        .to_str()
        .unwrap_or("unknown.could not convert path to string")
        .to_string();
    // metadata
    if !filepath.exists() {
        return Err(UtilesError::FileDoesNotExist(fspath));
    }
    if !filepath.is_file() {
        return Err(UtilesError::NotAFile(fspath));
    }
    let db = Connection::open(filepath)?;
    Ok(db)
}

pub fn pragma_index_list_all_tables(
    conn: &Connection,
) -> RusqliteResult<HashMap<String, Vec<PragmaIndexListRow>>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_schema WHERE type='table'")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?;
    let tables = rows.collect::<RusqliteResult<Vec<String>>>()?;
    let mut index_map = HashMap::new();
    for table in tables {
        let rows = pragma_index_list(conn, &table)?;
        index_map.insert(table, rows);
    }
    Ok(index_map)
}

pub fn application_id(conn: &Connection) -> RusqliteResult<u32> {
    let mut stmt = conn.prepare("PRAGMA application_id")?;
    let mut rows = stmt.query([])?;
    let row = rows
        .next()?
        .expect("'PRAGMA application_id' -- should return row but did not");
    let app_id: u32 = row.get(0)?;
    Ok(app_id)
}

pub fn journal_mode(conn: &Connection) -> RusqliteResult<String> {
    let mut stmt = conn.prepare("PRAGMA journal_mode")?;
    let mut rows = stmt.query([])?;
    let row = rows
        .next()?
        .expect("'PRAGMA journal_mode' -- should return row but did not");
    let jm: String = row.get(0)?;
    Ok(jm)
}

pub fn magic_number(conn: &Connection) -> RusqliteResult<u32> {
    application_id(conn)
}

pub fn query_db_fspath(conn: &Connection) -> RusqliteResult<Option<String>> {
    let rows = pragma_database_list(conn)?;
    let row = rows.iter().find_map(|r| {
        if r.name == "main" {
            Some(r.file.clone())
        } else {
            None
        }
    });
    Ok(row)
}

pub fn is_empty_db(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare("SELECT COUNT(*) FROM sqlite_schema")?;
    let rows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows == 0_i64)
}

pub fn vacuum(conn: &Connection) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("VACUUM")?;
    let r = stmt.execute([])?;
    Ok(r)
}

pub fn vacuum_into(conn: &Connection, dst: String) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("VACUUM INTO ?")?;
    let r = stmt.execute([dst])?;
    Ok(r)
}

pub fn analyze(conn: &Connection) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("ANALYZE")?;
    let r = stmt.execute([])?;
    Ok(r)
}
