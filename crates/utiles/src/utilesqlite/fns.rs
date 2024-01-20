use imagesize;
use log::error;
use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, Result};

use utiles_core::tile_type::tiletype_str;

pub fn add_function_ut_tiletype(db: &Connection) -> Result<()> {
    db.create_scalar_function(
        "ut_tiletype",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");
            // assert arg is blob
            let blob = ctx.get_raw(0).as_blob()?;
            let tt = tiletype_str(blob);
            Ok(tt)
        },
    )
}

/// Scalar function to return the size of a tile if it is a square image.
///
/// Returns -1 if the image is not square.
///
/// Returns NULL if not an image or errors... could do more here
pub fn add_function_ut_tilesize(db: &Connection) -> Result<()> {
    db.create_scalar_function(
        "ut_tilesize",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");
            let blob = ctx.get_raw(0).as_blob()?;

            let size: Option<i64> = match imagesize::blob_size(blob) {
                Ok(imgsize) => {
                    if imgsize.width == imgsize.height {
                        Some(imgsize.width as i64)
                    } else {
                        // -1 is there is a problem and img is not square
                        Some(-1)
                    }
                }
                Err(e) => {
                    error!("error getting image size: {}", e);
                    None
                }
            };
            let v = match size {
                Some(s) => rusqlite::types::Value::Integer(s),
                None => rusqlite::types::Value::Null,
            };
            Ok(v)
        },
    )
}

pub fn add_ut_functions(db: &Connection) -> Result<()> {
    add_function_ut_tiletype(db)?;
    add_function_ut_tilesize(db)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn repo_root() -> std::path::PathBuf {
        // recurse up until we find a dir called "test-data"
        let mut p = std::env::current_dir().unwrap();
        let mut count = 0;
        loop {
            if count > 5 {
                panic!("too many recursions");
            }
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
            ("0.vector.pbfz", "pbfgz", None),
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
        let expected = vec![
            "gif", "jpg", "json", "pbf", "pbfgz", "png", "unknown", "webp",
        ];
        assert_eq!(distinct_rows, expected);
        Ok(())
    }
}
