use rusqlite::Connection;

use crate::sqlite::RusqliteResult;

pub fn attach_db(conn: &Connection, db: &str, as_: &str) -> RusqliteResult<()> {
    let sql = format!("ATTACH DATABASE '{}' AS '{}'", db, as_);
    conn.execute(sql.as_str(), [])?;
    Ok(())
}

pub fn detach_db(conn: &Connection, db: &str) -> RusqliteResult<()> {
    let sql = format!("DETACH DATABASE '{}'", db);
    conn.execute(sql.as_str(), [])?;
    Ok(())
}
