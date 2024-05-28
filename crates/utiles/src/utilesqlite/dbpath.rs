use crate::errors::UtilesResult;
use crate::UtilesError;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct DbPath {
    /// the filesystem path to the mbtiles file
    pub fspath: String,
    /// the filename of the mbtiles file (filesystem path basename)
    pub filename: String,
}

impl std::fmt::Display for DbPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fspath)
    }
}

impl DbPath {
    pub fn new(fspath: &str) -> Self {
        let p = PathBuf::from(fspath);
        let filename = p
            .file_name()
            .expect("DbPath::new: invalid path, could not get filename from path");
        DbPath {
            fspath: fspath.to_string(),
            filename: filename
                .to_str()
                .expect(
                    "DbPath::new: invalid path, could not convert filename to string",
                )
                .to_string(),
        }
    }
    pub fn memory() -> Self {
        DbPath {
            fspath: ":memory:".to_string(),
            filename: ":memory:".to_string(),
        }
    }
}

// try from for dbpath

pub fn pathlike2dbpath<P: AsRef<std::path::Path>>(p: P) -> UtilesResult<DbPath> {
    debug!("pathlike2dbpath: {:?}", p.as_ref());
    let fspath = p
        .as_ref()
        .to_str()
        .ok_or(UtilesError::InvalidFspath(
            "pathlike2dbpath: invalid path".to_string(),
        ))?
        .to_string();
    let filename = p
        .as_ref()
        .file_name()
        .ok_or(UtilesError::InvalidFspath(
            "pathlike2dbpath: invalid filename".to_string(),
        ))?
        .to_str()
        .ok_or(UtilesError::InvalidFspath(
            "pathlike2dbpath: invalid filename".to_string(),
        ))?
        .to_string();
    Ok(DbPath { fspath, filename })
}

// impl<P: AsRef<std::path::Path>> From<P> for DbPath {
//     fn from(p: P) -> Self {
//         debug!("DbPath::from: {:?}", p.as_ref());
//         pathlike2dbpath(p).unwrap()
//     }
// }

pub trait DbPathTrait {
    fn db_path(&self) -> &DbPath;
    fn filepath(&self) -> &str {
        &self.db_path().fspath
    }

    fn filename(&self) -> &str {
        &self.db_path().filename
    }
}
