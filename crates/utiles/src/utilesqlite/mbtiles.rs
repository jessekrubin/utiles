use std::error::Error;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result as RusqliteResult};
use tilejson::TileJSON;

use tracing::{debug, error};

use utiles_core::bbox::BBox;
use utiles_core::errors::UtilesCoreResult;
use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;
use utiles_core::mbutiles::MinZoomMaxZoom;
use utiles_core::tile_data_row::TileData;
use utiles_core::{yflip, LngLat, Tile, TileLike, UtilesCoreError};

use crate::errors::UtilesResult;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::insert_strategy::InsertStrategy;
use crate::utilesqlite::mbtstats::MbtilesZoomStats;
use crate::utilesqlite::sql_schemas::MBTILES_FLAT_SQLITE_SCHEMA;
use crate::utilesqlite::squealite::{open_existing, Sqlike3};

#[derive(Debug, Default)]
pub enum MbtilesType {
    #[default]
    Flat,
    Hash,
    Norm,
}

pub struct Mbtiles {
    pub(crate) conn: Connection,
}

impl Sqlike3 for Mbtiles {
    fn conn(&self) -> &Connection {
        &self.conn
    }
}

impl Mbtiles {
    pub fn open<P: AsRef<Path>>(path: P) -> RusqliteResult<Self> {
        // if it is ':memory:' then open_in_memory
        if path.as_ref().to_str().unwrap() == ":memory:" {
            return Ok(Mbtiles::open_in_memory());
        }
        let path = path.as_ref().to_owned();
        let conn_res = Connection::open(path);
        match conn_res {
            Ok(c) => Ok(Mbtiles { conn: c }),
            Err(e) => Err(e),
        }
    }

    pub fn open_with_flags<P: AsRef<Path>>(
        path: P,
        flags: rusqlite::OpenFlags,
    ) -> Self {
        let path = path.as_ref().to_owned();
        let conn = Connection::open_with_flags(path, flags).unwrap();
        Mbtiles { conn }
    }

    #[must_use]
    pub fn open_in_memory() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        Mbtiles { conn }
    }

    pub fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let c = open_existing(path);
        match c {
            Ok(c) => Ok(Mbtiles::from_conn(c)),
            Err(e) => Err(e),
        }
        // let path = path.as_ref().to_owned();
        // if !path.is_file() {
        //     let emsg = format!("File does not exist: {}", path.to_str().unwrap());
        //     return Err(UtilesError::FileDoesNotExist(path.to_str().unwrap().to_string()));
        // }
        // let mbt_res = Mbtiles::open(path).map_err(|e| {
        //     let emsg = format!("Error opening mbtiles file: {}", e);
        //     UtilesError::Unknown(emsg)
        // });
        // match mbt_res {
        //     Ok(mbt) => Ok(mbt),
        //     Err(e) => Err(e),
        // }
    }

    // pub fn vacuum(&self) -> RusqliteResult<usize> {
    //     vacuum(&self.conn)
    // }
    //
    // pub fn analyze(&self) -> RusqliteResult<usize> {
    //     analyze(&self.conn)
    // }

    pub fn init_flat_mbtiles(&mut self) -> RusqliteResult<()> {
        init_flat_mbtiles(&mut self.conn)
    }

    pub fn create(
        filepath: &str,
        mbtype: Option<MbtilesType>,
    ) -> UtilesCoreResult<Self> {
        let res = create_mbtiles_file(filepath, mbtype.unwrap_or_default())?;
        Ok(Mbtiles { conn: res })
    }

    pub fn from_conn(conn: Connection) -> Mbtiles {
        Mbtiles { conn }
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn metadata(&self) -> RusqliteResult<Vec<MbtilesMetadataRow>> {
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
        metadata: &Vec<MbtilesMetadataRow>,
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
                serde_json::to_string(&rows).unwrap()
            );
            // return the first one
            let row = rows.first().unwrap();
            Ok(Some(row.value.clone()))
        } else {
            let row = rows.first().unwrap();
            Ok(Some(row.value.clone()))
        }
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

    pub fn bbox(&self) -> Result<BBox, Box<dyn Error>> {
        let bounding = self.tilejson()?.bounds;
        match bounding {
            Some(bounds) => Ok(BBox::new(
                bounds.left,
                bounds.bottom,
                bounds.right,
                bounds.top,
            )),
            None => Err("Error parsing metadata to TileJSON: no data available".into()),
        }
        // convert boundsd to BBox
    }

    pub fn contains(&self, lnglat: LngLat) -> Result<bool, Box<dyn Error>> {
        let bbox = self.bbox()?;
        let contains = bbox.contains_lnglat(lnglat);
        // return false if not ok
        Ok(contains)
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

    pub fn metadata_table_name_is_primary_key(&self) -> RusqliteResult<bool> {
        let mut stmt = self.conn.prepare("SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name='metadata' AND sql LIKE '%PRIMARY KEY%'")?;
        let nrows = stmt.query_row([], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        })?;
        Ok(nrows == 1_i64)
    }

    pub fn zoom_levels(&self) -> RusqliteResult<Vec<u8>> {
        zoom_levels(&self.conn)
    }

    pub fn minzoom_maxzoom(&self) -> RusqliteResult<Option<MinZoomMaxZoom>> {
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

    pub fn insert_tiles_flat(
        &mut self,
        tiles: &Vec<TileData>,
    ) -> RusqliteResult<usize> {
        insert_tiles_flat_mbtiles(&mut self.conn, tiles, Some(InsertStrategy::Ignore))
    }
    pub fn magic_number(&self) -> RusqliteResult<u32> {
        self.application_id()
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

impl From<&Path> for Mbtiles {
    fn from(path: &Path) -> Self {
        let conn = Connection::open(path).unwrap();
        Mbtiles { conn }
    }
}

// =====================================================================
// QUERY FUNCTIONS ~ QUERY FUNCTIONS ~ QUERY FUNCTIONS ~ QUERY FUNCTIONS
// =====================================================================

/// return a vector of MbtilesMetadataRow structs
pub fn mbtiles_metadata(conn: &Connection) -> RusqliteResult<Vec<MbtilesMetadataRow>> {
    let mut stmt = conn.prepare_cached("SELECT name, value FROM metadata")?;
    let mdata = stmt
        .query_map([], |row| {
            Ok(MbtilesMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesMetadataRow>, rusqlite::Error>>();

    match mdata {
        Ok(mdata) => Ok(mdata),
        Err(e) => {
            error!("Error getting metadata: {}", e);
            Err(e)
        }
    }
}

/// Return true/false if metadata table has a unique index on 'name'
pub fn has_unique_index_on_metadata(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='metadata' AND name='name'")?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn metadata_table_name_is_primary_key(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name='metadata' AND sql LIKE '%PRIMARY KEY%'")?;
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
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='view' AND name='tiles'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_tiles_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name='tiles'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_metadata_table(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name='metadata'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
}

pub fn has_metadata_view(connection: &Connection) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='view' AND name='metadata'",
    )?;
    let nrows = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(nrows == 1_i64)
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

pub fn tile_exists(connection: &Connection, tile: Tile) -> RusqliteResult<bool> {
    let mut stmt = connection.prepare_cached("SELECT COUNT(*) FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3")?;
    let rows = stmt.query_row(params![tile.z, tile.x, tile.flipy()], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    Ok(rows == 1_i64)
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

pub fn create_mbtiles_file(
    fspath: &str,
    mbtype: MbtilesType,
) -> UtilesCoreResult<Connection> {
    let mut conn = Connection::open(fspath).map_err(|e| {
        let emsg = format!("Error opening mbtiles file: {}", e);
        UtilesCoreError::Unknown(emsg)
    })?;
    match mbtype {
        MbtilesType::Flat => {
            let r = init_flat_mbtiles(&mut conn);
            match r {
                Ok(_) => Ok(conn),
                Err(e) => {
                    error!("Error creating flat mbtiles file: {}", e);
                    let emsg = format!("Error creating flat mbtiles file: {}", e);
                    Err(UtilesCoreError::Unknown(emsg))
                }
            }

            // match r {
            //     Ok(_) => Ok(()),
            //     Err(e) => {
            //         error!("Error creating flat mbtiles file: {}", e);
            //         let emsg = format!("Error creating flat mbtiles file: {}", e);
            //         Err(UtilesError::Unknown(emsg))
            //     }
            // }
        }
        _ => Err(UtilesCoreError::Unimplemented(
            "create_mbtiles_file: only flat mbtiles is implemented".to_string(),
        )),
    }

    // // Ok(
    //     conn
    // )
}

pub fn insert_tile_flat_mbtiles(
    conn: &mut Connection,
    tile: Tile,
    data: Vec<u8>,
) -> RusqliteResult<usize> {
    let mut stmt = conn.prepare_cached("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)")?;
    let r = stmt.execute(params![tile.z, tile.x, tile.flipy(), data])?;
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
        let statement = format!("{} INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)", insert_clause);
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
) -> RusqliteResult<Vec<MbtilesMetadataRow>> {
    let mut stmt =
        conn.prepare_cached("SELECT name, value FROM metadata WHERE name=?1")?;
    let mdata = stmt
        .query_map(params![name], |row| {
            Ok(MbtilesMetadataRow {
                name: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesMetadataRow>, rusqlite::Error>>()?;
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
    metadata: &Vec<MbtilesMetadataRow>,
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
                MbtilesMetadataRow {
                    name: "minzoom".to_string(),
                    value: minmax.minzoom.to_string(),
                },
                MbtilesMetadataRow {
                    name: "maxzoom".to_string(),
                    value: minmax.maxzoom.to_string(),
                },
            ],
        ),
        None => Ok(0),
    }
}

pub fn zoom_stats(conn: &Connection) -> RusqliteResult<Vec<MbtilesZoomStats>> {
    // total tiles
    let mut stmt = conn.prepare_cached(
        "SELECT zoom_level, COUNT(*), MIN(tile_row), MAX(tile_row), MIN(tile_column), MAX(tile_column)
         FROM tiles
         GROUP BY zoom_level"
    ).unwrap();

    let rows = stmt
        .query_map([], |row| {
            let zoom: u32 = row.get(0)?;

            let ntiles: i64 = row.get(1)?;
            let min_tile_column: i64 = row.get(4)?;
            let max_tile_column: i64 = row.get(5)?;
            let min_tile_row: i64 = row.get(2)?;
            let max_tile_row: i64 = row.get(3)?;
            // flip the stuff
            let ymin = yflip(max_tile_row as u32, zoom.try_into().unwrap());
            let ymax = yflip(min_tile_row as u32, zoom.try_into().unwrap());
            Ok(MbtilesZoomStats {
                zoom,
                ntiles,
                xmin: min_tile_column as u32,
                xmax: max_tile_column as u32,
                ymin,
                ymax,
            })
        })?
        .collect::<RusqliteResult<Vec<MbtilesZoomStats>, rusqlite::Error>>()
        .unwrap();
    Ok(rows)
}
