use std::error::Error;
use std::path::Path;

use rusqlite::{Connection, Result as RusqliteResult};
use tilejson::TileJSON;
use tracing::error;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;
use utiles::mbtiles::{metadata2tilejson, MinZoomMaxZoom};

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
    }

    pub fn tj(&self) -> Result<TileJSON, Box<dyn Error>> {
        self.tilejson()
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
    pub fn has_unique_index_on_metadata(&self) -> RusqliteResult<bool> {
        has_unique_index_on_metadata(&self.conn)
    }

    pub fn zoom_levels(&self) -> RusqliteResult<Vec<u32>> {
        zoom_levels(&self.conn)
    }

    pub fn minzoom_maxzoom(&self) -> RusqliteResult<MinZoomMaxZoom> {
        minzoom_maxzoom(&self.conn)
    }

    pub fn application_id(&self) -> RusqliteResult<u32> {
        // PRAGMA application_id
        let mut stmt = self.conn.prepare("PRAGMA application_id")?;
        let mut rows = stmt.query([])?;
        let row = rows.next()?.unwrap();
        let app_id: u32 = row.get(0)?;
        Ok(app_id)
    }

    pub fn magic_number(&self) -> RusqliteResult<u32> {
        self.application_id()
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

pub fn zoom_levels(conn: &Connection) -> RusqliteResult<Vec<u32>> {
    let mut stmt = conn.prepare("SELECT DISTINCT zoom_level FROM tiles")?;
    let zoom_levels = stmt
        .query_map([], |row| row.get(0))?
        .collect::<RusqliteResult<Vec<u32>, rusqlite::Error>>()?;
    Ok(zoom_levels)
}

pub fn minzoom_maxzoom(conn: &Connection) -> RusqliteResult<MinZoomMaxZoom> {
    let mut stmt = conn.prepare("SELECT MIN(zoom_level), MAX(zoom_level) FROM (SELECT DISTINCT zoom_level FROM tiles)")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.unwrap();
    let minzoom: u32 = row.get(0)?;
    let maxzoom: u32 = row.get(1)?;
    let mm =
        MinZoomMaxZoom::new(minzoom.try_into().unwrap(), maxzoom.try_into().unwrap());
    Ok(mm)
}

pub fn has_tiles_table_or_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='tiles'",
    )?;
    let mut rows = stmt.query([])?;
    let mut count = 0;
    while let Some(_row) = rows.next().unwrap() {
        count += 1;
    }
    Ok(count == 1)
}

pub fn has_tiles_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='view' AND name='tiles'")?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

pub fn has_tiles_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='tiles'",
    )?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

pub fn has_metadata_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='metadata'",
    )?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

pub fn has_metadata_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type='view' AND name='metadata'",
    )?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

pub fn has_metadata_table_or_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare("SELECT name FROM sqlite_master WHERE name='metadata' AND (type='table' OR type='view')")?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

pub fn is_mbtiles(connection: &Connection) -> RusqliteResult<bool> {
    // check for both metadata table/view and tiles table/view
    let has_metadata = has_metadata_table_or_view(connection)?;
    let has_tiles = has_tiles_table_or_view(connection)?;
    Ok(has_metadata && has_tiles)
}
