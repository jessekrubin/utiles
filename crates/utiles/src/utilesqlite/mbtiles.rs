use std::error::Error;
use std::path::Path;

use crate::sqlite::RusqliteResult;
use rusqlite::{params, Connection, OptionalExtension};
use tilejson::TileJSON;
use tracing::{debug, error, warn};

use utiles_core::bbox::BBox;
use utiles_core::tile_data_row::TileData;
use utiles_core::{yflip, LngLat, Tile, TileLike, UtilesCoreError};

use crate::errors::UtilesResult;
use crate::mbt::query::query_mbtiles_type;
use crate::mbt::{
    MbtMetadataRow, MbtType, MbtilesStats, MbtilesZoomStats, MinZoomMaxZoom,
};
use crate::sqlite::InsertStrategy;
use crate::sqlite::{
    application_id, open_existing, pragma_index_info, pragma_index_list,
    pragma_table_list, query_db_fspath, Sqlike3,
};
use crate::sqlite_utiles::add_ut_functions;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::dbpath::{pathlike2dbpath, DbPath};
use crate::utilesqlite::hash_types::HashType;

use crate::utilesqlite::sql_schemas::MBTILES_FLAT_SQLITE_SCHEMA;
use crate::UtilesError;

pub struct Mbtiles {
    pub dbpath: DbPath,
    pub(crate) conn: Connection,
}

impl Sqlike3 for Mbtiles {
    fn conn(&self) -> &Connection {
        &self.conn
    }
}

impl Mbtiles {
    pub fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        // if it is ':memory:' then open_in_memory
        let dbpath = pathlike2dbpath(path)?;
        let conn_res = Connection::open(&dbpath.fspath);
        match conn_res {
            Ok(c) => Ok(Mbtiles { conn: c, dbpath }),
            Err(e) => Err(UtilesError::RusqliteError(e)),
        }
    }

    pub fn open_with_flags<P: AsRef<Path>>(
        path: P,
        flags: rusqlite::OpenFlags,
    ) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(path)?;
        let conn = Connection::open_with_flags(&dbpath.fspath, flags)?;
        Ok(Mbtiles { dbpath, conn })
    }

    pub fn open_in_memory() -> UtilesResult<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Mbtiles {
            conn,
            dbpath: DbPath::memory(),
        })
    }

    pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let c = open_existing(path);
        match c {
            Ok(c) => Mbtiles::from_conn(c),
            Err(e) => Err(e),
        }
    }

    pub fn db_filesize(&self) -> UtilesResult<u64> {
        let pth = Path::new(&self.dbpath.fspath);
        debug!("pth: {:?}", pth);
        let metadata = pth.metadata()?;
        debug!("metadata: {:?}", metadata);
        // file size
        let filesize = metadata.len();
        debug!("filesize: {:?}", filesize);

        Ok(metadata.len())
    }

    pub fn init_flat_mbtiles(&mut self) -> RusqliteResult<()> {
        init_flat_mbtiles(&mut self.conn)
    }
    // pub fn open

    pub fn create<P: AsRef<Path>>(
        path: P,
        mbtype: Option<MbtType>,
    ) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(&path)?;
        let res = create_mbtiles_file(&path, &mbtype.unwrap_or_default())?;
        Ok(Mbtiles { conn: res, dbpath })
    }

    pub fn from_conn(conn: Connection) -> UtilesResult<Mbtiles> {
        let guessed_fspath = query_db_fspath(&conn);
        let dbpath = match guessed_fspath {
            Ok(Some(fspath)) => pathlike2dbpath(fspath)?,
            Ok(None) => DbPath::memory(),
            Err(e) => {
                error!("Error guessing fspath: {}", e);
                DbPath::memory()
            }
        };
        Ok(Mbtiles { dbpath, conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn metadata(&self) -> RusqliteResult<Vec<MbtMetadataRow>> {
        mbtiles_metadata(&self.conn)
    }

    pub fn metadata_set(&self, name: &str, value: &str) -> RusqliteResult<usize> {
        metadata_set(&self.conn, name, value)
    }

    pub fn metadata_delete(&self, name: &str) -> RusqliteResult<usize> {
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM metadata WHERE name=?1")?;
        let r = stmt.execute(params![name])?;
        Ok(r)
    }

    pub fn metadata_set_from_vec(
        &self,
        metadata: &Vec<MbtMetadataRow>,
    ) -> RusqliteResult<usize> {
        metadata_set_many(&self.conn, metadata)
    }

    pub fn metadata_get(&self, name: &str) -> RusqliteResult<Option<String>> {
        let rows = metadata_get(&self.conn, name)?;
        if rows.is_empty() {
            return Ok(None);
        }
        if rows.len() > 1 {
            error!(
                "metadata has more than one row for name: {} - {}",
                name,
                serde_json::to_string(&rows)
                    .expect("metadata_get: error serializing metadata rows")
            );
            // return the first one
            let row = rows.first();
            match row {
                Some(row) => Ok(Some(row.value.clone())),
                None => Ok(None),
            }
        } else {
            let row = rows.first();
            match row {
                None => Ok(None),
                Some(row) => {
                    let value = row.value.clone();
                    Ok(Some(value))
                }
            }
        }
    }

    pub fn tilejson(&self) -> UtilesResult<TileJSON> {
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

    pub fn bbox(&self) -> UtilesResult<BBox> {
        let bounding = self.tilejson()?.bounds;
        match bounding {
            Some(bounds) => Ok(BBox::new(
                bounds.left,
                bounds.bottom,
                bounds.right,
                bounds.top,
            )),
            None => Err(UtilesError::ParsingError(
                "Error parsing metadata to BBox: no bounds".into(),
            )),
        }
    }

    pub fn contains(&self, lnglat: LngLat) -> UtilesResult<bool> {
        let bbox = self.bbox()?;
        let contains = bbox.contains_lnglat(&lnglat);
        // return false if not ok
        Ok(contains)
    }

    pub fn tj(&self) -> UtilesResult<TileJSON> {
        self.tilejson()
    }

    pub fn from_filepath(fspath: &str) -> UtilesResult<Mbtiles> {
        let dbpath = pathlike2dbpath(fspath)?;
        let conn = Connection::open(fspath)?;
        Ok(Mbtiles { dbpath, conn })
    }

    pub fn from_filepath_str(fspath: &str) -> Result<Mbtiles, Box<dyn Error>> {
        Mbtiles::from_filepath(fspath).map_err(std::convert::Into::into)
    }

    // check that 'metadata' table exists and has a unique index on 'name'
    pub fn has_unique_index_on_metadata(&self) -> RusqliteResult<bool> {
        has_unique_index_on_metadata(&self.conn)
    }

    pub fn metadata_table_name_is_primary_key(&self) -> RusqliteResult<bool> {
        metadata_table_name_is_primary_key(&self.conn)
    }

    pub fn zoom_levels(&self) -> RusqliteResult<Vec<u8>> {
        zoom_levels(&self.conn)
    }

    pub fn minzoom_maxzoom(&self) -> RusqliteResult<Option<MinZoomMaxZoom>> {
        minzoom_maxzoom(&self.conn)
    }

    pub fn application_id(&self) -> RusqliteResult<u32> {
        application_id(&self.conn)
    }

    pub fn insert_tiles_flat(
        &mut self,
        tiles: &Vec<TileData>,
    ) -> RusqliteResult<usize> {
        insert_tiles_flat_mbtiles(&mut self.conn, tiles, Some(InsertStrategy::Ignore))
    }

    pub fn insert_tile_flat<T: TileLike>(
        &mut self,
        tile: &T,
        datal: &[u8],
    ) -> RusqliteResult<usize> {
        insert_tile_flat_mbtiles::<T>(&mut self.conn, tile.tile(), datal)
    }

    pub fn magic_number(&self) -> RusqliteResult<u32> {
        self.application_id()
    }

    pub fn query_mbt_type(&self) -> UtilesResult<MbtType> {
        query_mbtiles_type(&self.conn)
    }

    pub fn mbt_stats(&self) -> UtilesResult<MbtilesStats> {
        let query_ti = std::time::Instant::now();
        let filesize = self.db_filesize()?;
        debug!("Started zoom_stats query");
        let page_count = self.pragma_page_count()?;
        let page_size = self.pragma_page_size()?;
        let freelist_count = self.pragma_freelist_count()?;
        let zoom_stats = self.zoom_stats()?;
        let query_dt = query_ti.elapsed();
        debug!("Finished zoom_stats query in {:?}", query_dt);
        let mbt_type = self.query_mbt_type()?;
        if zoom_stats.is_empty() {
            return Ok(MbtilesStats {
                filesize,
                mbtype: mbt_type,
                page_count,
                page_size,
                freelist_count,
                ntiles: 0,
                minzoom: None,
                maxzoom: None,
                nzooms: 0,
                zooms: vec![],
            });
        }

        let minzoom = zoom_stats.iter().map(|r| r.zoom).min();
        let maxzoom = zoom_stats.iter().map(|r| r.zoom).max();
        let minzoom_u8: Option<u8> = minzoom
            .map(|minzoom| minzoom.try_into().expect("Error converting minzoom to u8"));
        let maxzoom_u8: Option<u8> = maxzoom
            .map(|maxzoom| maxzoom.try_into().expect("Error converting maxzoom to u8"));
        Ok(MbtilesStats {
            ntiles: zoom_stats.iter().map(|r| r.ntiles).sum(),
            filesize,
            mbtype: mbt_type,
            page_count,
            page_size,
            freelist_count,
            minzoom: minzoom_u8,
            maxzoom: maxzoom_u8,
            nzooms: zoom_stats.len() as u32,
            zooms: zoom_stats,
        })
    }

    pub fn zoom_stats(&self) -> RusqliteResult<Vec<MbtilesZoomStats>> {
        zoom_stats(&self.conn)
    }

    pub fn tiles_count(&self) -> RusqliteResult<usize> {
        tiles_count(&self.conn)
    }

    pub fn tiles_count_at_zoom(&self, zoom: u8) -> RusqliteResult<usize> {
        tiles_count_at_zoom(&self.conn, zoom)
    }

    pub fn update_metadata_minzoom_from_tiles(&self) -> RusqliteResult<usize> {
        update_metadata_minzoom_from_tiles(&self.conn)
    }

    pub fn update_metadata_maxzoom_from_tiles(&self) -> RusqliteResult<usize> {
        update_metadata_maxzoom_from_tiles(&self.conn)
    }

    pub fn update_metadata_minzoom_maxzoom_from_tiles(&self) -> RusqliteResult<usize> {
        update_metadata_minzoom_maxzoom_from_tiles(&self.conn)
    }
}

impl<P: AsRef<Path>> From<P> for Mbtiles {
    // TODO: fix uses of this
    #[allow(clippy::unwrap_used)]
    fn from(p: P) -> Self {
        Mbtiles::open_existing(p).unwrap()
    }
}

// =========================================================================
// SQLITE FUNCTIONS ~ SQLITE FUNCTIONS ~ SQLITE FUNCTIONS ~ SQLITE FUNCTIONS
// =========================================================================
pub fn add_sqlite_hashes(conn: &Connection) -> RusqliteResult<()> {
    sqlite_hashes::register_hash_functions(conn)
}

pub fn register_utiles_sqlite_functions(conn: &Connection) -> RusqliteResult<()> {
    add_ut_functions(conn)
}

pub fn add_functions(conn: &Connection) -> RusqliteResult<()> {
    add_sqlite_hashes(conn)?;
    register_utiles_sqlite_functions(conn)
}

// =====================================================================
// QUERY FUNCTIONS ~ QUERY FUNCTIONS ~ QUERY FUNCTIONS ~ QUERY FUNCTIONS
// =====================================================================

/// return a vector of `MbtilesMetadataRow` structs
pub fn mbtiles_metadata(conn: &Connection) -> RusqliteResult<Vec<MbtMetadataRow>> {
    let mut stmt = conn.prepare_cached("SELECT name, value FROM metadata")?;
    let mdata = stmt
        .query_map([], |row| {
            Ok(MbtMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtMetadataRow>, rusqlite::Error>>();

    match mdata {
        Ok(mdata) => Ok(mdata),
        Err(e) => {
            error!("Error getting metadata: {}", e);
            Err(e)
        }
    }
}

pub fn mbtiles_metadata_row(
    conn: &Connection,
    name: &str,
) -> RusqliteResult<Option<MbtMetadataRow>> {
    let mut stmt =
        conn.prepare_cached("SELECT name, value FROM metadata WHERE name=?1")?;
    let mdata = stmt
        .query_row(params![name], |row| {
            Ok(MbtMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .optional()?;
    Ok(mdata)
}

/// Return true/false if metadata table has a unique index on 'name'
pub fn has_unique_index_on_metadata(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_schema WHERE type='index' AND tbl_name='metadata' AND name='name'")?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn metadata_table_name_is_primary_key(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(name) FROM sqlite_schema WHERE type='table' AND name='metadata' AND sql LIKE '%PRIMARY KEY%'")?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn zoom_levels(conn: &Connection) -> RusqliteResult<Vec<u8>> {
    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT zoom_level FROM tiles ORDER BY zoom_level ASC",
    )?;
    let zoom_levels = stmt
        .query_map([], |row| row.get(0))?
        .collect::<RusqliteResult<Vec<u32>, rusqlite::Error>>()?;
    // convert to u8
    let zoom_levels = zoom_levels.iter().map(|z| *z as u8).collect::<Vec<u8>>();
    Ok(zoom_levels)
}

pub fn minzoom(conn: &Connection) -> RusqliteResult<Option<u32>> {
    let mut stmt = conn.prepare_cached("SELECT MIN(zoom_level) FROM tiles")?;
    let minzoom: Option<u32> = stmt.query_row([], |row| row.get(0)).optional()?;
    Ok(minzoom)
}

pub fn maxzoom(conn: &Connection) -> RusqliteResult<Option<u32>> {
    let mut stmt = conn.prepare_cached("SELECT MAX(zoom_level) FROM tiles")?;
    let maxzoom: Option<u32> = stmt.query_row([], |row| row.get(0)).optional()?;
    Ok(maxzoom)
}

pub fn minzoom_maxzoom(conn: &Connection) -> RusqliteResult<Option<MinZoomMaxZoom>> {
    let mut stmt = conn.prepare("SELECT MIN(zoom_level), MAX(zoom_level) FROM (SELECT DISTINCT zoom_level FROM tiles)")?;
    let minmax: Option<(Option<u32>, Option<u32>)> = stmt
        .query_row([], |row| {
            let minzoom: Option<u32> = row.get(0)?;
            let maxzoom: Option<u32> = row.get(1)?;
            Ok((minzoom, maxzoom))
        })
        .optional()?;
    match minmax {
        Some((minzoom, maxzoom)) => match (minzoom, maxzoom) {
            (Some(minzoom), Some(maxzoom)) => {
                let minzoom_u8 = minzoom as u8;
                let maxzoom_u8 = maxzoom as u8;
                Ok(Some(MinZoomMaxZoom {
                    minzoom: minzoom_u8,
                    maxzoom: maxzoom_u8,
                }))
            }
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

pub fn has_tiles_table_or_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_schema WHERE name='tiles' AND (type='table' OR type='view')",
    )?;
    let mut rows = stmt.query([])?;
    let mut count = 0;
    while let Some(_row) = rows.next()? {
        count += 1;
    }
    Ok(count == 1)
}

pub fn has_tiles_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_schema WHERE type='view' AND name='tiles'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_tiles_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_schema WHERE type='table' AND name='tiles'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_metadata_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_schema WHERE type='table' AND name='metadata'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_metadata_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_schema WHERE type='view' AND name='metadata'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_metadata_table_or_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare("SELECT name FROM sqlite_schema WHERE name='metadata' AND (type='table' OR type='view')")?;
    let nrows = stmt.query([]).iter().count();
    Ok(nrows == 1)
}

// TODO: make actually robust...
pub fn has_zoom_row_col_autoindex(connection: &Connection) -> RusqliteResult<bool> {
    let tables_list = pragma_table_list(connection)?;
    let tables = tables_list
        .iter()
        .filter(|t| t.type_ == "table")
        .collect::<Vec<_>>();

    for table in tables {
        let indexes = pragma_index_list(connection, &table.name);
        match indexes {
            Ok(indexes) => {
                let unique_indexes =
                    indexes.iter().filter(|i| i.unique).collect::<Vec<_>>();
                for index in unique_indexes {
                    let index_info = pragma_index_info(connection, &index.name)?;
                    let index_info = index_info
                        .iter()
                        .filter(|i| {
                            i.name == "zoom_level"
                                || i.name == "tile_column"
                                || i.name == "tile_row"
                        })
                        .collect::<Vec<_>>();
                    if index_info.len() == 3 {
                        return Ok(true);
                    }
                }
            }
            Err(e) => {
                error!("Error getting indexes: {}", e);
                return Err(e);
            }
        }
    }

    warn!("No index/autoindex found for zoom_level, tile_column, tile_row");
    Ok(false)
}

pub fn has_zoom_row_col_index(connection: &Connection) -> RusqliteResult<bool> {
    // check that there is an index in the db that indexes columns named zoom_level, tile_column, tile_row

    let q = "
        SELECT
            idx.name AS index_name,
            tbl.name AS table_name,
            idx.sql AS index_sql
        FROM
            sqlite_schema AS tbl
        JOIN
            sqlite_schema AS idx ON tbl.name = idx.tbl_name
        WHERE
            tbl.type IN ('table', 'view') AND
            tbl.name = 'tiles' AND
            idx.type = 'index' AND
            (
                idx.sql LIKE '%zoom_level%' OR
                idx.sql LIKE '%tile_column%' OR
                idx.sql LIKE '%tile_row%'
            );
    ";
    let mut stmt = connection.prepare(q)?;
    let nrows = stmt
        .query_map([], |row| {
            let index_name: String = row.get(0)?;
            let table_name: String = row.get(1)?;
            let index_sql: String = row.get(2)?;
            Ok((index_name, table_name, index_sql))
        })?
        .collect::<RusqliteResult<Vec<(String, String, String)>>>()?;
    match nrows.len() {
        0 => {
            let check_autoindex = has_zoom_row_col_autoindex(connection)?;
            if !check_autoindex {
                warn!("No index/autoindex found for zoom_level, tile_column, tile_row");
            }
            Ok(check_autoindex)
        }
        _ => Ok(true),
    }
    // Ok(nrows.len() > 0)
}

pub fn is_mbtiles(connection: &Connection) -> RusqliteResult<bool> {
    // check for both metadata table/view and tiles table/view
    let has_metadata = has_metadata_table_or_view(connection)?;
    let has_tiles = has_tiles_table_or_view(connection)?;
    Ok(has_metadata && has_tiles)
}

pub fn query_zxy(
    connection: &Connection,
    z: u8,
    x: u32,
    y: u32,
) -> RusqliteResult<Option<Vec<u8>>> {
    let mut stmt = connection.prepare_cached("SELECT tile_data FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3")?;
    let tile_data: Option<Vec<u8>> = stmt
        .query_row(params![z, x, y], |row| row.get(0))
        .optional()?;
    Ok(tile_data)
}

pub fn query_tile<T: TileLike>(
    connection: &Connection,
    tile: &T,
) -> RusqliteResult<Option<Vec<u8>>> {
    query_zxy(connection, tile.z(), tile.x(), tile.y())
}

pub fn tile_exists<T: TileLike>(
    connection: &Connection,
    tile: &T,
) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare_cached("SELECT COUNT(*) FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3")?;
    let rows = stmt.query_row(params![tile.z(), tile.x(), tile.flipy()], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows == 1_i64)
}

pub fn tiles_is_empty(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare_cached("SELECT * FROM tiles LIMIT 1")?;
    let rows = stmt
        .query_row([], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        })
        .optional()?;
    Ok(rows.is_none())
}

pub fn tiles_count(connection: &Connection) -> RusqliteResult<usize> {
    let mut stmt = connection.prepare_cached("SELECT COUNT(*) FROM tiles")?;
    let rows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows as usize)
}

pub fn tiles_count_at_zoom(connection: &Connection, zoom: u8) -> RusqliteResult<usize> {
    let mut stmt =
        connection.prepare_cached("SELECT COUNT(*) FROM tiles WHERE zoom_level=?1")?;
    let rows = stmt.query_row(params![zoom], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows as usize)
}

pub fn init_mbtiles_hash(conn: &mut Connection) -> RusqliteResult<()> {
    let script = "
        -- metadata table
        CREATE TABLE metadata
        (
            name  TEXT NOT NULL PRIMARY KEY,
            value TEXT
        );
        -- unique index on name
        CREATE UNIQUE INDEX metadata_name_index ON metadata (name);
        -- tiles table
        CREATE TABLE tiles
        (
            zoom_level  INTEGER NOT NULL,
            tile_column INTEGER NOT NULL,
            tile_row    INTEGER NOT NULL,
            tile_data   BLOB
        );
        -- unique index on zoom_level, tile_column, tile_row
        CREATE UNIQUE INDEX tile_index ON tiles (zoom_level, tile_column, tile_row);
    ";
    let tx = conn.transaction();
    match tx {
        Ok(tx) => {
            let script_res = tx.execute_batch(script);
            debug!("init_flat_mbtiles: script_res: {:?}", script_res);
            let r = tx.commit();
            debug!("init_flat_mbtiles: r: {:?}", r);
            Ok(())
        }
        Err(e) => {
            error!("Error creating transaction: {}", e);
            Err(e)
        }
    }
}

pub fn init_flat_mbtiles(conn: &mut Connection) -> RusqliteResult<()> {
    let tx = conn.transaction();
    match tx {
        Ok(tx) => {
            let script_res = tx.execute_batch(MBTILES_FLAT_SQLITE_SCHEMA);
            debug!("init_flat_mbtiles: script_res: {:?}", script_res);
            let r = tx.commit();
            debug!("init_flat_mbtiles: r: {:?}", r);
            Ok(())
        }
        Err(e) => {
            error!("Error creating transaction: {}", e);
            Err(e)
        }
    }
}

pub fn create_mbtiles_file<P: AsRef<Path>>(
    fspath: P,
    mbtype: &MbtType,
) -> UtilesResult<Connection> {
    let mut conn = Connection::open(fspath).map_err(|e| {
        let emsg = format!("Error opening mbtiles file: {e}");
        UtilesCoreError::Unknown(emsg)
    })?;
    match mbtype {
        MbtType::Flat => {
            let r = init_flat_mbtiles(&mut conn);
            match r {
                Ok(()) => Ok(conn),
                Err(e) => {
                    error!("Error creating flat mbtiles file: {}", e);
                    let emsg = format!("Error creating flat mbtiles file: {e}");
                    Err(UtilesError::Unknown(emsg))
                }
            }
        }
        _ => Err(UtilesError::Unimplemented(
            "create_mbtiles_file: only flat mbtiles is implemented".to_string(),
        )),
    }
}

pub fn insert_tile_flat_mbtiles<T: TileLike>(
    conn: &mut Connection,
    tile: Tile,
    data: &[u8],
) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)")?;
    let r = stmt.execute(params![tile.z(), tile.x(), tile.flipy(), data])?;
    Ok(r)
}

pub fn insert_tiles_flat_mbtiles(
    conn: &mut Connection,
    tiles: &Vec<TileData>,
    insert_strategy: Option<InsertStrategy>,
) -> RusqliteResult<usize> {
    let tx = conn.transaction().expect("Error creating transaction");

    let insert_strat = insert_strategy.unwrap_or_default();
    let insert_clause = insert_strat.sql_prefix();
    // TODO - use batch insert
    // let batch_size = 999;
    // let chunk_size = tiles.len() / batch_size;

    // scope so that stmt is not borrowed when tx.commit() is called
    let mut naff: usize = 0;
    {
        let statement = format!("{insert_clause} INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)");
        let mut stmt = tx.prepare_cached(&statement)?;
        for tile in tiles {
            let r = stmt.execute(params![
                tile.xyz.z,
                tile.xyz.x,
                tile.xyz.flipy(),
                tile.data
            ])?;
            naff += r;
        }
    }
    tx.commit().expect("Error committing transaction");
    Ok(naff)
}

pub fn metadata_get(
    conn: &Connection,
    name: &str,
) -> RusqliteResult<Vec<MbtMetadataRow>> {
    let mut stmt =
        conn.prepare_cached("SELECT name, value FROM metadata WHERE name=?1")?;
    let mdata = stmt
        .query_map(params![name], |row| {
            Ok(MbtMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtMetadataRow>, rusqlite::Error>>()?;
    Ok(mdata)
}

pub fn metadata_set(
    conn: &Connection,
    name: &str,
    value: &str,
) -> RusqliteResult<usize> {
    // Use an UPSERT statement to insert or update as necessary
    let sql = "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)";
    let mut stmt = conn.prepare_cached(sql)?;
    let r = stmt.execute(params![name, value])?;
    Ok(r)
}

pub fn metadata_set_many(
    conn: &Connection,
    metadata: &Vec<MbtMetadataRow>,
) -> RusqliteResult<usize> {
    // Use an UPSERT statement to insert or update as necessary
    let sql = "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)";
    let mut stmt = conn.prepare_cached(sql)?;
    let mut naff: usize = 0;
    for md in metadata {
        let r = stmt.execute(params![md.name, md.value])?;
        naff += r;
    }
    Ok(naff)
}

pub fn update_metadata_minzoom_from_tiles(conn: &Connection) -> RusqliteResult<usize> {
    let minzoom = minzoom(conn)?;
    match minzoom {
        Some(minzoom) => {
            let mut stmt = conn.prepare_cached(
                "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)",
            )?;
            let r = stmt.execute(params!["minzoom", minzoom])?;
            Ok(r)
        }
        None => Ok(0),
    }
}

pub fn update_metadata_maxzoom_from_tiles(conn: &Connection) -> RusqliteResult<usize> {
    let maxzoom = maxzoom(conn)?;
    match maxzoom {
        Some(maxzoom) => {
            let mut stmt = conn.prepare_cached(
                "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)",
            )?;
            let r = stmt.execute(params!["maxzoom", maxzoom])?;
            Ok(r)
        }
        None => Ok(0),
    }
}

pub fn update_metadata_minzoom_maxzoom_from_tiles(
    conn: &Connection,
) -> RusqliteResult<usize> {
    let minmax = minzoom_maxzoom(conn)?;
    match minmax {
        Some(minmax) => metadata_set_many(
            conn,
            &vec![
                MbtMetadataRow {
                    name: "minzoom".to_string(),
                    value: minmax.minzoom.to_string(),
                },
                MbtMetadataRow {
                    name: "maxzoom".to_string(),
                    value: minmax.maxzoom.to_string(),
                },
            ],
        ),
        None => Ok(0),
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn zoom_stats(conn: &Connection) -> RusqliteResult<Vec<MbtilesZoomStats>> {
    // total tiles
    let mut stmt = conn.prepare_cached(
        "SELECT zoom_level, COUNT(*), MIN(tile_row), MAX(tile_row), MIN(tile_column), MAX(tile_column), SUM(OCTET_LENGTH(tile_data)) as nbytes
         FROM tiles
         GROUP BY zoom_level"
    )?;

    let rows = stmt
        .query_map([], |row| {
            let zoom: u32 = row.get(0)?;

            let ntiles = row.get(1)?;
            let min_tile_column: i64 = row.get(4)?;
            let max_tile_column: i64 = row.get(5)?;
            let min_tile_row: i64 = row.get(2)?;
            let max_tile_row: i64 = row.get(3)?;
            // flip the stuff
            let zu8 = zoom as u8;
            let ymin = yflip(max_tile_row as u32, zu8);
            let ymax = yflip(min_tile_row as u32, zu8);
            let nbytes: u64 = row.get(6)?;
            let nbytes_avg: f64 = nbytes as f64 / ntiles as f64;
            Ok(MbtilesZoomStats {
                zoom,
                ntiles,
                xmin: min_tile_column as u32,
                xmax: max_tile_column as u32,
                ymin,
                ymax,
                nbytes,
                nbytes_avg,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesZoomStats>, rusqlite::Error>>()?;
    Ok(rows)
}

// =================================================================
// queries with sqlite extension functions
// =================================================================

pub fn query_distinct_tiletype_fast(conn: &Connection) -> RusqliteResult<Vec<String>> {
    let mut stmt = conn.prepare(
        //     for each zoom get 1 random row and then get the distinct tiletypes
        "SELECT DISTINCT ut_tiletype(tile_data) AS tile_type FROM (SELECT zoom_level, tile_data FROM tiles GROUP BY zoom_level) GROUP BY zoom_level;"
    )?;

    let tile_format: Vec<String> =
        stmt.query_map([], |row| row.get(0))?
            .collect::<RusqliteResult<Vec<String>, rusqlite::Error>>()?;
    Ok(tile_format)
}

pub fn query_distinct_tiletype(conn: &Connection) -> RusqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT ut_tiletype(tile_data) FROM tiles")?;
    let tile_format: Vec<String> =
        stmt.query_map([], |row| row.get(0))?
            .collect::<RusqliteResult<Vec<String>, rusqlite::Error>>()?;
    Ok(tile_format)
}

// =================================================================
// HASH FUNCTIONS ~ HASH FUNCTIONS ~ HASH FUNCTIONS ~ HASH FUNCTIONS
// =================================================================
fn mbt_agg_tile_hash_query(hash_type: HashType) -> String {
    let sql = format!(
        "SELECT coalesce(
            {hash_type}_concat_hex(
                cast(zoom_level AS text),
                cast(tile_column AS text),
                cast(tile_row AS text),
                tile_data
                ORDER BY zoom_level, tile_column, tile_row),
            {hash_type}_hex(''))
        FROM tiles"
    );
    sql
}

pub fn mbt_agg_tiles_hash(
    conn: &Connection,
    hash_type: HashType,
) -> RusqliteResult<String> {
    let mut stmt = conn.prepare_cached(mbt_agg_tile_hash_query(hash_type).as_str())?;
    let agg_tiles_hash_str: String = stmt.query_row([], |row| row.get(0))?;
    Ok(agg_tiles_hash_str)
}

pub fn mbt_agg_tiles_hash_md5(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Md5)
}

pub fn mbt_agg_tiles_hash_fnv1a(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Fnv1a)
}

pub fn mbt_agg_tiles_hash_xxh32(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Xxh32)
}

pub fn mbt_agg_tiles_hash_xxh64(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Xxh64)
}

pub fn mbt_agg_tiles_hash_xxh3_64(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Xxh3_64)
}

pub fn mbt_agg_tiles_hash_xxh3_128(conn: &Connection) -> RusqliteResult<String> {
    mbt_agg_tiles_hash(conn, HashType::Xxh3_128)
}
