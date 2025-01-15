//! utiles-sqlite ~ sqlite extension function(s) for utiles
//!
//! Adds the following functions:
//!   - `ut_tiletype(blob)`   - returns the tile type of the blob
//!   - `ut_tilesize(blob)`   - returns the size of raster tile or None
//!   - `xxh3_int(blob|str)`  - returns xxh3 hash as `i64` big-endian view
//!   - `xxh64_int(blob|str)` - returns xxh64 hash as `i64` big-endian view
use crate::sqlite_utiles::base64::add_function_base64_encode;
use crate::sqlite_utiles::hash_int::{
    add_function_fnv_i64, add_function_xxh3_i64, add_function_xxh64_i64,
};
use crate::sqlite_utiles::tilesize::add_function_ut_tilesize;
use crate::sqlite_utiles::tiletype::add_function_ut_tiletype;
use rusqlite::{Connection, Result};
use tracing::debug;

mod base64;
mod hash_int;
mod tilesize;
mod tiletype;

pub fn add_ut_functions(db: &Connection) -> Result<()> {
    debug!("registering sqlite-utiles functions...");
    add_function_ut_tiletype(db)?;
    add_function_ut_tilesize(db)?;
    add_function_base64_encode(db)?;

    add_function_xxh3_i64(db)?;
    add_function_xxh64_i64(db)?;
    add_function_fnv_i64(db)?;
    debug!("registered sqlite-utiles functions!");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use rusqlite::params;

    type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

    #[allow(clippy::panic)]
    fn repo_root() -> std::path::PathBuf {
        // recurse up until we find a dir called "test-data"
        let mut p = std::env::current_dir().unwrap();
        let mut count = 0;
        loop {
            assert!(count <= 5, "too many recursions");
            if p.join("test-data").exists() && p.join("test-data").is_dir() {
                break;
            }
            p = p.parent().unwrap().to_path_buf();
            count += 1;
        }
        p
    }

    fn test_data_dirpath() -> std::path::PathBuf {
        let r = repo_root();
        r.join("test-data")
    }

    #[test]
    fn test_ut_tiletype() -> Result<(), BoxError> {
        let db = rusqlite::Connection::open_in_memory()?;
        super::add_ut_functions(&db)?;

        let test_data_dirpath = test_data_dirpath();
        let test_data_tile_types_dirpath = test_data_dirpath.join("tile-types");

        // create table with filename and data columns
        db.execute(
            "CREATE TABLE tiles (
                filename TEXT,
                data BLOB
            )",
            [],
        )?;

        // for each file in test-data/tile-types
        for entry in std::fs::read_dir(test_data_tile_types_dirpath)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            let bytes = std::fs::read(&path)?;
            // insert into table
            db.execute(
                "INSERT INTO tiles (filename, data) VALUES (?, ?)",
                params![filename, bytes],
            )?;
        }
        // query the table
        let mut stmt = db.prepare("SELECT filename, ut_tiletype(data) as tiletype, ut_tilesize(data) as tilesize FROM tiles ORDER BY filename")?;
        let rows = stmt
            .query_map(params![], |row| {
                let filename: String = row.get(0)?;
                let tiletype: String = row.get(1)?;
                let tilesize: Option<i32> = row.get(2)?;
                let row = (filename, tiletype, tilesize);
                Ok(row)
            })?
            .collect::<Result<Vec<(String, String, Option<i32>)>, rusqlite::Error>>()?;
        let expected = [
            ("0.gif", "gif", Some(256)),
            ("0.jpeg", "jpg", Some(256)),
            ("0.png", "png", Some(256)),
            ("0.vector.pbf", "pbf", None),
            ("0.vector.pbf.gz", "pbf.gz", None),
            ("0.vector.pbf.zlib", "pbf.zlib", None),
            ("0.vector.pbf.zst", "pbf.zst", None),
            ("0.webp", "webp", Some(256)),
            ("gif-990x1050.gif", "gif", Some(-1)),
            ("jpg-640x400.jpg", "jpg", Some(-1)),
            ("png-640x400.png", "png", Some(-1)),
            ("tile-arr.json", "json", None),
            ("tile-obj.json", "json", None),
            ("tux.webp", "webp", Some(-1)),
            ("tux_alpha.webp", "webp", Some(-1)),
            ("unknown.txt", "unknown", None),
            ("webp-550x368.webp", "webp", Some(-1)),
        ];
        for (i, (filename, tiletype, tilesize)) in rows.iter().enumerate() {
            assert_eq!(filename, expected[i].0);
            assert_eq!(tiletype, expected[i].1);
            assert_eq!(tilesize, &expected[i].2);
        }
        let mut distinct_stmt =
            db.prepare("SELECT DISTINCT ut_tiletype(data) FROM tiles")?;
        let mut distinct_rows = distinct_stmt
            .query_map(params![], |row| {
                let tiletype: String = row.get(0)?;
                Ok(tiletype)
            })?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
        distinct_rows.sort();
        let mut expected = vec![
            "gif", "jpg", "json", "pbf", "pbf.gz", "pbf.zlib", "pbf.zst", "png",
            "unknown", "webp",
        ];
        expected.sort_unstable();
        assert_eq!(distinct_rows, expected);
        Ok(())
    }

    #[test]
    fn test_ut_base64() -> Result<(), BoxError> {
        let db = rusqlite::Connection::open_in_memory()?;
        super::add_ut_functions(&db)?;
        // maketablwe and insert data
        db.execute(
            "CREATE TABLE data (
                id INTEGER PRIMARY KEY,
                data BLOB
            )",
            [],
        )?;
        let data = b"hello world~";
        db.execute("INSERT INTO data (data) VALUES (?)", params![data])?;
        let mut stmt = db.prepare("SELECT base64_encode(data) FROM data")?;
        let rows = stmt
            .query_map(params![], |row| {
                let b64: String = row.get(0)?;
                Ok(b64)
            })?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;
        assert_eq!(rows.len(), 1);
        let expected = "aGVsbG8gd29ybGR+";
        assert_eq!(rows[0], expected);
        Ok(())
    }
}
