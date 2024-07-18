use crate::errors::UtilesResult;
use crate::fs_async::file_exists;
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
    #[must_use]
    pub fn new(fspath: &str) -> Self {
        // let p = PathBuf::from(fspath);
        pathlike2dbpath(fspath).map_or(
            DbPath {
                fspath: "unknown".to_string(),
                filename: "unknown".to_string(),
            },
            |a| a,
        )
    }

    #[must_use]
    pub fn memory() -> Self {
        DbPath {
            fspath: ":memory:".to_string(),
            filename: ":memory:".to_string(),
        }
    }

    #[must_use]
    pub fn fspath_exists(&self) -> bool {
        PathBuf::from(&self.fspath).exists()
    }

    #[must_use]
    pub async fn fspath_exists_async(&self) -> bool {
        file_exists(&self.fspath).await
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
