use std::error::Error;
use std::path::Path;

use rusqlite::{Connection, Result as RusqliteResult};
use tilejson::TileJSON;
use tracing::error;
use utiles::mbtiles::metadata2tilejson;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;

pub struct Mbtiles {
    conn: Connection,
}

impl Mbtiles {
    pub fn from_conn(conn: Connection) -> Mbtiles {
        Mbtiles { conn }
    }

    pub fn metadata(&self) -> RusqliteResult<Vec<MbtilesMetadataRow>> {
        mbtiles_metadata(&self.conn)
    }

    pub fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
        let metadata = self.metadata()?;
        let tj = metadata2tilejson(metadata);
        match tj {
            Ok(t) => Ok(t),
            Err(e) => {
                error!("Error parsing metadata to TileJSON: {}", e);
                Err(e)
            }
        }
        // return Ok(tj);
    }

    pub fn from_filepath(fspath: &str) -> RusqliteResult<Mbtiles> {
        let conn = Connection::open(fspath)?;
        let mbt = Mbtiles { conn };
        Ok(mbt)
    }

    pub fn from_filepath_str(fspath: &str) -> Result<Mbtiles, Box<dyn Error>> {
        let conn = Connection::open(fspath)?;
        let mbt = Mbtiles { conn };
        Ok(mbt)
    }


    // check that 'metadata' table exists and has a unique index on 'name'
    pub fn has_unique_index_on_metadata(&self) ->  RusqliteResult<bool>{
        has_unique_index_on_metadata(& self.conn)
    }
}

impl From<&Path> for Mbtiles {
    fn from(path: &Path) -> Self {
        let conn = Connection::open(path).unwrap();
        Mbtiles { conn }
    }
}

pub fn mbtiles_metadata(conn: &Connection) -> RusqliteResult<Vec<MbtilesMetadataRow>> {
    let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
    let mdata = stmt
        .query_map([], |row| {
            Ok(MbtilesMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesMetadataRow>, rusqlite::Error>>()?;
    Ok(mdata)
}

// check that 'metadata' table exists and has a unique index on 'name'
pub fn has_unique_index_on_metadata(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='metadata' AND name='name'")?;
    let mut rows = stmt.query([])?;
    let mut count = 0;
    while let Some(_row) = rows.next()? {
        count += 1;
    }
    let res = count == 1;
    Ok(res)
}
