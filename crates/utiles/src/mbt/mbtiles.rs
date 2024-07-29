use indoc::indoc;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::path::Path;
use tilejson::TileJSON;
use tracing::{debug, error, warn};

use utiles_core::bbox::BBox;
use utiles_core::constants::MBTILES_MAGIC_NUMBER;
use utiles_core::tile_data_row::TileData;
use utiles_core::{flipy, yflip, LngLat, Tile, TileLike};

use crate::errors::UtilesResult;
use crate::mbt::query::{
    create_mbtiles_indexes_norm, create_mbtiles_tables_norm,
    create_mbtiles_tiles_view_norm, create_metadata_table_pk, create_tiles_index_flat,
    create_tiles_index_hash, create_tiles_table_flat, create_tiles_table_hash,
    create_tiles_view_hash, query_mbtiles_type,
};
use crate::mbt::{
    query_mbt_stats, MbtMetadataRow, MbtType, MbtilesMetadataJson, MbtilesStats,
    MbtilesZoomStats, MinZoomMaxZoom,
};
use crate::sqlite::{
    application_id, open_existing, pragma_index_info, pragma_index_list,
    pragma_table_list, query_db_fspath, Sqlike3,
};
use crate::sqlite::{application_id_set, InsertStrategy};
use crate::sqlite::{pathlike2dbpath, DbPath};
use crate::sqlite::{pragma_encoding_set, RusqliteResult};
use crate::sqlite_utiles::add_ut_functions;
use crate::utilejson::metadata2tilejson;
use crate::UtilesError;

#[derive(Debug)]
pub struct Mbtiles {
    pub dbpath: DbPath,
    pub(crate) conn: Connection,
}

impl Sqlike3 for Mbtiles {
    fn conn(&self) -> &Connection {
        &self.conn
    }

    fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

impl Mbtiles {
    pub fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        // if it is ':memory:' then open_in_memory
        let dbpath = pathlike2dbpath(path)?;
        let conn_res = Connection::open(&dbpath.fspath)?;
        Ok(Mbtiles {
            conn: conn_res,
            dbpath,
        })
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

    pub fn open_new<P: AsRef<Path>>(
        path: P,
        mbtype: Option<MbtType>,
    ) -> UtilesResult<Self> {
        let mbtype = mbtype.unwrap_or(MbtType::Flat);

        // make sure the path don't exist
        let dbpath = pathlike2dbpath(path)?;
        if dbpath.fspath_exists() {
            Err(UtilesError::PathExistsError(dbpath.fspath))
        } else {
            debug!("Creating new mbtiles file with client: {}", dbpath);
            Mbtiles::create(&dbpath.fspath, Some(mbtype))
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

    pub fn query_zxy(&self, z: u8, x: u32, y: u32) -> RusqliteResult<Option<Vec<u8>>> {
        query_zxy(&self.conn, z, x, y)
    }

    pub fn query_tile<T: TileLike>(&self, tile: &T) -> RusqliteResult<Option<Vec<u8>>> {
        query_tile(&self.conn, tile)
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

    pub fn metadata_set_many(
        &self,
        metadata: &Vec<MbtMetadataRow>,
    ) -> RusqliteResult<usize> {
        metadata_set_many(&self.conn, metadata)
    }

    pub fn metadata_delete(&self, name: &str) -> RusqliteResult<usize> {
        metadata_delete(&self.conn, name)
    }

    pub fn metadata_set_from_vec(
        &self,
        metadata: &Vec<MbtMetadataRow>,
    ) -> RusqliteResult<usize> {
        metadata_set_many(&self.conn, metadata)
    }

    pub fn metadata_get(&self, name: &str) -> UtilesResult<Option<String>> {
        let rows = metadata_get(&self.conn, name)?;
        if rows.is_empty() {
            return Ok(None);
        }
        if rows.len() > 1 {
            error!(
                "metadata has more than one row for name: {} - {:?}",
                name, rows,
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

    pub fn metadata_json(&self) -> UtilesResult<MbtilesMetadataJson> {
        metadata_json(&self.conn).map_err(|e| e.into())
    }

    // pub fn metadata_update(&self, name: &str, value: &str) -> RusqliteResult<usize> {
    //     metadata_update(&self.conn, name, value)
    // }

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

    // pub fn insert_tiles_flat_tuple(
    //     &mut self,
    //     tiles: &Vec<(Tile, Vec<u8>>,
    // ) -> RusqliteResult<usize> {
    //     insert_tiles_flat_mbtiles(&mut self.conn, tiles, Some(InsertStrategy::Ignore))
    // }

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
        query_mbtiles_type(&self.conn).map_err(|e| e.into())
    }

    pub fn mbt_stats(&self, full: Option<bool>) -> UtilesResult<MbtilesStats> {
        query_mbt_stats(&self.conn, full)
    }

    pub fn zoom_stats(&self, full: bool) -> RusqliteResult<Vec<MbtilesZoomStats>> {
        if full {
            zoom_stats_full(&self.conn)
        } else {
            zoom_stats(&self.conn)
        }
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

/// Return a Map<String, Value> of metadata
pub fn metadata_json(conn: &Connection) -> RusqliteResult<MbtilesMetadataJson> {
    let mdata = mbtiles_metadata(conn)?;
    let md_json = MbtilesMetadataJson::from(&mdata);
    Ok(md_json)
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
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_schema WHERE type='index' AND tbl_name='metadata' AND sql LIKE '%UNIQUE%'")?;
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
    Ok(false)
}

pub fn has_zoom_row_col_index(connection: &Connection) -> RusqliteResult<bool> {
    // check that there is an index in the db that indexes columns named zoom_level, tile_column, tile_row

    let q = indoc! {"
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
    "};
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
    let yup = flipy(y, z);
    let tile_data: Option<Vec<u8>> = stmt
        .query_row(params![z, x, yup], |row| row.get(0))
        .optional()?;
    Ok(tile_data)
}

pub fn has_zxy(connection: &Connection, z: u8, x: u32, y: u32) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare_cached("SELECT COUNT(*) FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3 LIMIT 1")?;
    let yup = flipy(y, z);
    let count: i64 = stmt.query_row(params![z, x, yup], |row| row.get(0))?;
    Ok(count == 1)
}
pub fn has_tile<T: TileLike>(
    connection: &Connection,
    tile: &T,
) -> RusqliteResult<bool> {
    has_zxy(connection, tile.z(), tile.x(), tile.y())
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
    has_tile(connection, tile)
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
    let tx = conn.transaction()?;
    create_metadata_table_pk(&tx)?;
    create_tiles_table_hash(&tx, false)?;
    create_tiles_index_hash(&tx)?;
    create_tiles_view_hash(&tx)?;
    tx.commit()
}

pub fn init_flat_mbtiles(conn: &mut Connection) -> RusqliteResult<()> {
    let tx = conn.transaction()?;
    debug!("creating metadata table");
    create_metadata_table_pk(&tx)?;
    debug!("creating tiles table");
    create_tiles_table_flat(&tx, false)?;
    create_tiles_index_flat(&tx)?;
    tx.commit()
}

pub fn init_mbtiles_normalized(conn: &mut Connection) -> RusqliteResult<()> {
    let tx = conn.transaction()?;
    create_metadata_table_pk(&tx)?;
    create_mbtiles_tables_norm(&tx)?;
    create_mbtiles_indexes_norm(&tx)?;
    create_mbtiles_tiles_view_norm(&tx)?;
    tx.commit()
}

pub fn init_mbtiles(conn: &mut Connection, mbt: &MbtType) -> UtilesResult<()> {
    application_id_set(conn, MBTILES_MAGIC_NUMBER)?;
    pragma_encoding_set(conn, "UTF-8")?;
    let r: UtilesResult<()> = match mbt {
        MbtType::Flat => init_flat_mbtiles(conn).map_err(|e| e.into()),
        MbtType::Hash => init_mbtiles_hash(conn).map_err(|e| e.into()),
        MbtType::Norm => init_mbtiles_normalized(conn).map_err(|e| e.into()),
        _ => {
            let emsg = format!("init_mbtiles: {mbt} not implemented");
            Err(UtilesError::Unimplemented(emsg))
        }
    };
    r
}

pub fn create_mbtiles_file<P: AsRef<Path>>(
    fspath: P,
    mbtype: &MbtType,
) -> UtilesResult<Connection> {
    let mut conn = Connection::open(fspath)?;
    init_mbtiles(&mut conn, mbtype)?;
    Ok(conn)
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
    let tx = conn.transaction()?;
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
    tx.commit()?;
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

pub fn metadata_delete(conn: &Connection, name: &str) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("DELETE FROM metadata WHERE name=?1")?;
    let r = stmt.execute(params![name])?;
    Ok(r)
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
pub fn zoom_stats_full(conn: &Connection) -> RusqliteResult<Vec<MbtilesZoomStats>> {
    // total tiles
    let mut stmt = conn.prepare_cached(
        r"
        SELECT
            zoom_level,
            COUNT(*) AS ntiles,
            MIN(tile_row) AS min_tile_row,
            MAX(tile_row) AS max_tile_row,
            MIN(tile_column) AS min_tile_column,
            MAX(tile_column) AS max_tile_column,
            SUM(OCTET_LENGTH(tile_data)) AS nbytes
        FROM
            tiles
        GROUP BY
            zoom_level
    ",
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
                nbytes: Some(nbytes),
                nbytes_avg: Some(nbytes_avg),
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesZoomStats>, rusqlite::Error>>()?;
    Ok(rows)
}

#[allow(clippy::cast_precision_loss)]
pub fn zoom_stats(conn: &Connection) -> RusqliteResult<Vec<MbtilesZoomStats>> {
    // total tiles
    let mut stmt = conn.prepare_cached(indoc! {
    r"
        SELECT
            zoom_level,
            COUNT(*) AS ntiles,
            MIN(tile_row) AS min_tile_row,
            MAX(tile_row) AS max_tile_row,
            MIN(tile_column) AS min_tile_column,
            MAX(tile_column) AS max_tile_column
        FROM
            tiles
        GROUP BY
            zoom_level
    ",
    })?;

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

            Ok(MbtilesZoomStats {
                zoom,
                ntiles,
                xmin: min_tile_column as u32,
                xmax: max_tile_column as u32,
                ymin,
                ymax,
                nbytes: None,
                nbytes_avg: None,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesZoomStats>, rusqlite::Error>>()?;
    Ok(rows)
}

// =================================================================
// queries with sqlite extension functions
// =================================================================
pub fn query_distinct_tiletype_zoom_limit(
    conn: &Connection,
    zoom: u32,
    limit: u8,
) -> RusqliteResult<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        //     for each zoom get 1 random row and then get the distinct tiletypes
        "SELECT DISTINCT ut_tiletype(tile_data) FROM tiles WHERE zoom_level=?1 LIMIT ?2"
    )?;

    let tile_format: Vec<String> = stmt
        .query_map([zoom, u32::from(limit)], |row| row.get(0))?
        .collect::<RusqliteResult<Vec<String>, rusqlite::Error>>()?;
    Ok(tile_format)
}

pub fn query_distinct_tiletype_fast(
    conn: &Connection,
    min_max_zoom: MinZoomMaxZoom,
) -> RusqliteResult<Vec<String>> {
    let mut tile_types_set = HashSet::new();
    for z in min_max_zoom.minzoom..=min_max_zoom.maxzoom {
        let a = query_distinct_tiletype_zoom_limit(conn, u32::from(z), 10)?;
        for t in a {
            tile_types_set.insert(t);
        }
    }
    let tile_types_vec: Vec<String> = tile_types_set.into_iter().collect();
    Ok(tile_types_vec)
}

pub fn query_distinct_tiletype_limit(
    conn: &Connection,
    limit: u32,
) -> RusqliteResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT DISTINCT ut_tiletype(tile_data) FROM tiles LIMIT ?1")?;
    let tile_format: Vec<String> =
        stmt.query_map([limit], |row| row.get(0))?
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

pub fn query_distinct_tilesize_zoom_limit(
    conn: &Connection,
    zoom: u32,
    limit: u8,
) -> RusqliteResult<Vec<i64>> {
    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT ut_tilesize(tile_data) FROM tiles WHERE zoom_level=?1 LIMIT ?2",
    )?;

    let tile_format_opts: Vec<Option<i64>> = stmt
        .query_map([zoom, u32::from(limit)], |row| row.get(0))?
        .collect::<RusqliteResult<Vec<Option<i64>>, rusqlite::Error>>()?;
    let tile_format: Vec<i64> = tile_format_opts.iter().filter_map(|x| *x).collect();
    Ok(tile_format)
}

pub fn query_distinct_tilesize_fast(
    conn: &Connection,
    min_max_zoom: MinZoomMaxZoom,
) -> RusqliteResult<Vec<i64>> {
    let mut tile_types_set = HashSet::new();
    for z in min_max_zoom.minzoom..=min_max_zoom.maxzoom {
        let a = query_distinct_tilesize_zoom_limit(conn, u32::from(z), 10)?;
        for t in a {
            tile_types_set.insert(t);
        }
    }
    let tile_types_vec: Vec<i64> = tile_types_set.into_iter().collect();
    Ok(tile_types_vec)
}
