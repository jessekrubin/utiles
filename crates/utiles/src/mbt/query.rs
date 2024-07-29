use indoc::indoc;
use rusqlite::Connection;

use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::sqlite::RusqliteResult;

const IS_FLAT_MBTILES_QUERY: &str = include_str!("sql/is-flat-mbtiles-query.sql");
const IS_NORM_MBTILES_QUERY: &str = include_str!("sql/is-norm-mbtiles-query.sql");
const IS_HASH_MBTILES_QUERY: &str = include_str!("sql/is-hash-mbtiles-query.sql");
const IS_TIPPECANOE_MBTILES_QUERY: &str =
    include_str!("sql/is-tippecanoe-mbtiles-query.sql");

const IS_PLANETILER_MBTILES_QUERY: &str =
    include_str!("sql/is-planetiler-mbtiles-query.sql");

const METADATA_DUPLICATES_JSON_QUERY: &str =
    include_str!("sql/mbt-metadata-duplicates-json.sql");

pub fn is_tiles_with_hash(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_HASH_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_flat_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_FLAT_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_norm_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_NORM_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_tippecanoe_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_TIPPECANOE_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn is_planetiler_mbtiles(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(IS_PLANETILER_MBTILES_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn query_mbtiles_type(conn: &Connection) -> RusqliteResult<MbtType> {
    let is_tippecanoe = is_tippecanoe_mbtiles(conn)?;
    if is_tippecanoe {
        return Ok(MbtType::Tippecanoe);
    }
    let is_planetiler = is_planetiler_mbtiles(conn)?;
    if is_planetiler {
        return Ok(MbtType::Planetiler);
    }
    let is_norm = is_norm_mbtiles(conn)?;
    if is_norm {
        return Ok(MbtType::Norm);
    }
    let is_hash = is_tiles_with_hash(conn)?;

    if is_hash {
        return Ok(MbtType::Hash);
    }
    let is_flat = is_flat_mbtiles(conn)?;
    Ok(if is_flat {
        MbtType::Flat
    } else {
        MbtType::Unknown
    })
}

pub fn default_mbtiles_settings(conn: &Connection) -> UtilesResult<()> {
    // page size...
    {
        conn.execute_batch("PRAGMA page_size = 4096;")?;
    }
    // encoding UTF-8
    {
        conn.execute_batch("PRAGMA encoding = 'UTF-8';")?;
    }
    Ok(())
}

pub fn create_metadata_table_pk(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {
        "
            CREATE TABLE metadata (
                name TEXT PRIMARY KEY NOT NULL,
                value TEXT
            )
            "
    })?;
    Ok(())
}

pub fn create_metadata_table_nopk(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {
        "
            CREATE TABLE metadata (
                name TEXT NOT NULL,
                value TEXT
            )
            "
    })?;
    Ok(())
}
pub fn create_metadata_table_if_not_exists_nopk(
    conn: &Connection,
) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {
        "
            CREATE TABLE IF NOT EXISTS metadata (
                name TEXT NOT NULL,
                value TEXT
            )
            "
    })?;
    Ok(())
}

pub fn create_metadata_index(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS metadata_index ON metadata (name);",
    )?;
    Ok(())
}

pub fn drop_metadata_index(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch("DROP INDEX IF EXISTS metadata_index;")?;
    Ok(())
}

pub fn create_tiles_table_flat_pk(
    conn: &Connection,
    if_not_exists: bool,
) -> RusqliteResult<()> {
    if if_not_exists {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE IF NOT EXISTS tiles (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB,
                    PRIMARY KEY (zoom_level, tile_column, tile_row)
                )
                "
        })?;
    } else {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE tiles (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB,
                    PRIMARY KEY (zoom_level, tile_column, tile_row)
                )
                "
        })?;
    }
    Ok(())
}

pub fn create_tiles_table_flat(
    conn: &Connection,
    if_not_exists: bool,
) -> RusqliteResult<()> {
    if if_not_exists {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE IF NOT EXISTS tiles (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB
                )
                "
        })?;
    } else {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE tiles (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB
                )
                "
        })?;
    }
    Ok(())
}

pub fn create_tiles_table_hash(
    conn: &Connection,
    if_not_exists: bool,
) -> RusqliteResult<()> {
    if if_not_exists {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE IF NOT EXISTS tiles_with_hash (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB,
                    tile_hash   TEXT
                )
                "
        })?;
    } else {
        conn.execute_batch(indoc! {
            "
                CREATE TABLE tiles_with_hash (
                    zoom_level  INTEGER NOT NULL,
                    tile_column INTEGER NOT NULL,
                    tile_row    INTEGER NOT NULL,
                    tile_data   BLOB,
                    tile_hash   TEXT
                )
                "
        })?;
    }
    Ok(())
}

pub fn create_tiles_index_flat(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch("CREATE UNIQUE INDEX IF NOT EXISTS tile_index ON tiles (zoom_level, tile_column, tile_row);")?;
    Ok(())
}

pub fn create_tiles_index_hash(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch("CREATE UNIQUE INDEX IF NOT EXISTS tile_index ON tiles_with_hash (zoom_level, tile_column, tile_row);")?;
    Ok(())
}

pub fn create_tiles_view_hash(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(
        "CREATE VIEW tiles AS SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles_with_hash;",
    )?;
    Ok(())
}

pub fn create_mbtiles_tables_norm(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {
        "
            CREATE TABLE map (
                zoom_level  INTEGER NOT NULL,
                tile_column INTEGER NOT NULL,
                tile_row    INTEGER NOT NULL,
                tile_id     TEXT
            )
            "
    })?;
    conn.execute_batch(indoc! {
        "
            CREATE TABLE images (
                tile_id   TEXT NOT NULL,
                tile_data BLOB NOT NULL
            )
            "
    })?;
    Ok(())
}

pub fn create_mbtiles_indexes_norm(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(
        "CREATE UNIQUE INDEX map_index ON map (zoom_level, tile_column, tile_row);",
    )?;
    conn.execute_batch("CREATE UNIQUE INDEX images_id ON images (tile_id);")?;
    Ok(())
}

pub fn create_mbtiles_tiles_view_norm(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {
        "
            CREATE VIEW tiles AS
                SELECT
                    map.zoom_level   AS zoom_level,
                    map.tile_column  AS tile_column,
                    map.tile_row     AS tile_row,
                    images.tile_data AS tile_data
                FROM
                    map JOIN images
                    ON images.tile_id = map.tile_id;
            "
    })?;
    Ok(())
}

pub fn metadata_duplicates_json_query(conn: &Connection) -> RusqliteResult<String> {
    let mut stmt = conn.prepare(METADATA_DUPLICATES_JSON_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: String = row.get(0)?;
        Ok(a)
    })?;
    Ok(r)
}

pub fn fast_write_pragmas(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {r"
                PRAGMA synchronous = OFF;
                PRAGMA journal_mode = WAL;
                PRAGMA locking_mode = EXCLUSIVE;
                PRAGMA temp_store = MEMORY;
                PRAGMA cache_size = 100000;
                "
    })?;
    Ok(())
}

pub fn unfast_write_pragmas(conn: &Connection) -> RusqliteResult<()> {
    conn.execute_batch(indoc! {r"
            PRAGMA synchronous = NORMAL;
            PRAGMA journal_mode = DELETE;
            PRAGMA locking_mode = NORMAL;
            PRAGMA temp_store = DEFAULT;
            PRAGMA cache_size = 2000;
            "})?;
    Ok(())
}
