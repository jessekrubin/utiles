use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::sqlite::{
    PragmaIndexListRow, RusqliteResult, Sqlike3, pragma_database_list,
    pragma_index_list,
};

pub struct SqliteDb {
    pub conn: Connection,
}

impl Sqlike3 for SqliteDb {
    fn conn(&self) -> &Connection {
        &self.conn
    }

    fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

impl SqliteDb {
    pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let conn = open_existing(path)?;
        Ok(Self { conn })
    }
}

impl From<Connection> for SqliteDb {
    fn from(conn: Connection) -> Self {
        Self { conn }
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
