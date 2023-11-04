// #[derive(Debug)]
// pub struct Mbtiles<'a> {
//     conn: &'a mut rusqlite::Connection,
// }
use rusqlite::{Connection, Result};

pub struct MbtilesManager {
    conn: Option<Connection>,
}

#[derive(Debug)]
pub struct MbtilesMetadataRow {
    pub name: String,
    pub value: String,
}


pub fn mbtiles_metadata(conn: &rusqlite::Connection) -> Result<Vec<MbtilesMetadataRow>> {
    let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
    let mdata = stmt
        .query_map([], |row| {
            Ok(
                MbtilesMetadataRow {
                    name: row.get(0)?,
                    value: row.get(1)?,
                }
            )
        })?
        .collect::<Result<Vec<MbtilesMetadataRow>, rusqlite::Error>>()?;
    return Ok(mdata);
}

impl MbtilesManager {
    // Create a new instance of the MbtilesManager
    pub fn new() -> MbtilesManager {
        MbtilesManager { conn: None }
    }

    // Open a connection to the MBTiles SQLite database
    pub fn open(&mut self, path: &str) -> Result<()> {
        self.conn = Some(Connection::open(path)?);
        Ok(())
    }

    // Execute a query on the MBTiles database
    pub fn query<T, F>(&self, sql: &str, mut map_fn: F) -> Result<Vec<T>>
        where
            F: FnMut(&rusqlite::Row<'_>) -> Result<T>,
    {
        match &self.conn {
            Some(conn) => {
                let mut stmt = conn.prepare(sql)?;
                let rows = stmt.query_map([], |row| map_fn(row))?;
                rows.collect()
            }
            None => Err(rusqlite::Error::InvalidQuery),
        }
    }

    pub fn metadata(&self) -> Result<Vec<MbtilesMetadataRow>> {
        return mbtiles_metadata(self.conn.as_ref().unwrap());
        // let mut stmt = self.conn.as_ref().unwrap().prepare("SELECT name, value FROM metadata")?;
        // let rows = stmt.query_map([], |row| {
        //     Ok(
        //         MbtilesMetadataRow {
        //             name: row.get(0)?,
        //             value: row.get(1)?,
        //         }
        //     )
        // })?;
        // rows.collect()
    }

    // Close the connection to the MBTiles database
    pub fn close(&mut self) -> Result<()> {
        if let Some(conn) = self.conn.take() {
            conn.close().map_err(|(_, e)| e)
        } else {
            Ok(())
        }
    }
}

// fn main() {
//     let mut mbtiles_manager = MbtilesManager::new();
//
//     // Open the database connection
//     mbtiles_manager.open("path/to/your/mbtiles/database.mbtiles").unwrap();
//
//     // Execute a query
//     let result: Result<Vec<String>> = mbtiles_manager.query("SELECT name FROM some_table", |row| {
//         Ok(row.get(0)?)
//     });
//     match result {
//         Ok(rows) => {
//             for row in rows {
//                 println!("{}", row);
//             }
//         }
//         Err(err) => eprintln!("Query failed: {}", err),
//     }
//
//     // Close the database connection
//     mbtiles_manager.close().unwrap();
// }

// #[derive(Debug)]
// pub struct Mbtiles<'a> {
//     pub conn: &'a mut rusqlite::Connection,
// }
// #[derive(Debug)]
// pub struct MetadataRow {
//     pub name: String,
//     pub value: String,
// }
//
// impl Mbtiles<'_> {
// // impl Mbtiles {
//     pub fn metadata<'a>(&'a self) -> rusqlite::Result<Vec<MetadataRow>> {
//         // return all_metadata(self.conn);
//
//         let mut stmt = self.conn.prepare("SELECT name, value FROM metadata")?;
//         let mdata = stmt
//             .query_map([], |row| {
//                 Ok(
//                     MetadataRow {
//                         name: row.get(0)?,
//                         value: row.get(1)?,
//                     }
//                 )
//             })?
//             .collect::<rusqlite::Result<Vec<MetadataRow>>>();
//         return Ok(mdata?);
//     }
//
//     pub fn open<'a>(fspath: &str) -> rusqlite::Result<Mbtiles> {
//         let mut conn  = rusqlite::Connection::open(fspath)?;
//         let mbt = Mbtiles {
//             conn: &mut conn,
//         };
//
//         return Ok(mbt);
//
//     }
//
//     pub fn from_conn<'a>(conn: &mut rusqlite::Connection) -> Mbtiles {
//         Mbtiles {
//             conn: conn,
//         }
//     }
// }
//
//
// pub fn all_metadata (conn: &rusqlite::Connection) -> rusqlite::Result<Vec<MetadataRow>> {
//     let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
//     let mdata = stmt
//         .query_map([], |row| {
//             Ok(
//                 MetadataRow {
//                     name: row.get(0)?,
//                     value: row.get(1)?,
//                 }
//             )
//         })?
//         .collect::<rusqlite::Result<Vec<MetadataRow>>>();
//     return Ok(mdata?);
// }

// #[derive(Debug)]
// pub struct Mbtiles<'a> {
//     pub conn: &'a mut rusqlite::Connection,
// }
// #[derive(Debug)]
// pub struct MetadataRow {
//     pub name: String,
//     pub value: String,
// }
//
// impl Mbtiles<'_> {
//     // impl Mbtiles {
//     pub fn metadata<'a>(&'a self) -> rusqlite::Result<Vec<MetadataRow>> {
//         // return all_metadata(self.conn);
//
//         let mut stmt = self.conn.prepare("SELECT name, value FROM metadata")?;
//         let mdata = stmt
//             .query_map([], |row| {
//                 Ok(
//                     MetadataRow {
//                         name: row.get(0)?,
//                         value: row.get(1)?,
//                     }
//                 )
//             })?
//             .collect::<rusqlite::Result<Vec<MetadataRow>>>();
//         return Ok(mdata?);
//     }
//
//     pub fn open<'a>(fspath: &str) -> rusqlite::Result<Mbtiles> {
//         let mut conn  = rusqlite::Connection::open(fspath)?;
//         let mbt = Mbtiles {
//             conn: &mut conn,
//         };
//
//         return Ok(mbt);
//
//     }
//
//     pub fn from_conn<'a>(conn: &mut rusqlite::Connection) -> Mbtiles {
//         Mbtiles {
//             conn: conn,
//         }
//     }
// }
//
//
// pub fn all_metadata (conn: &rusqlite::Connection) -> rusqlite::Result<Vec<MetadataRow>> {
//     let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
//     let mdata = stmt
//         .query_map([], |row| {
//             Ok(
//                 MetadataRow {
//                     name: row.get(0)?,
//                     value: row.get(1)?,
//                 }
//             )
//         })?
//         .collect::<rusqlite::Result<Vec<MetadataRow>>>();
//     return Ok(mdata?);
// }
