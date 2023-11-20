use rusqlite::{Connection, Error};

pub fn open(path: &str) -> Result<Connection, Error> {
    let conn = Connection::open(path)?;
    Ok(conn)
}
