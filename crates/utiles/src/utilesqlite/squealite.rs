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
