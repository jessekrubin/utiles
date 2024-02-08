use std::path::Path;

use rusqlite::{Connection, Result as RusqliteResult};

use crate::errors::UtilesResult;
use crate::UtilesError;

pub trait Sqlike3 {
    fn conn(&self) -> &Connection;

    fn is_empty_db(&self) -> RusqliteResult<bool> {
        is_empty_db(self.conn())
    }

    fn vacuum(&self) -> RusqliteResult<usize> {
        vacuum(self.conn())
    }
    fn analyze(&self) -> RusqliteResult<usize> {
        analyze(self.conn())
    }
}

pub fn open(path: &str) -> RusqliteResult<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Connection> {
    let filepath = path.as_ref();
    if !filepath.exists() {
        return Err(UtilesError::FileDoesNotExist(
            path.as_ref().to_str().unwrap().to_string(),
        ));
    }
    let db = Connection::open(filepath)?;
    Ok(db)
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

pub fn application_id(conn: &Connection) -> RusqliteResult<u32> {
    let mut stmt = conn.prepare("PRAGMA application_id")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.unwrap();
    let app_id: u32 = row.get(0)?;
    Ok(app_id)
}

pub fn journal_mode(conn: &Connection) -> RusqliteResult<String> {
    let mut stmt = conn.prepare("PRAGMA journal_mode")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.unwrap();
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
    let mut stmt = connection.prepare("SELECT COUNT(*) FROM sqlite_master")?;
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

pub fn analyze(conn: &Connection) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("ANALYZE")?;
    let r = stmt.execute([])?;
    Ok(r)
}
