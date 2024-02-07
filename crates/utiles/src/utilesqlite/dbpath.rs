use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbPath {
    /// the filesystem path to the mbtiles file
    pub fspath: String,
    /// the filename of the mbtiles file (filesystem path basename)
    pub filename: String,
}

impl DbPath {
    pub fn new(fspath: &str) -> Self {
        let p = PathBuf::from(fspath);
        let filename = p.file_name().unwrap();
        let filename = filename.to_str().unwrap().to_string();
        DbPath {
            fspath: fspath.to_string(),
            filename,
        }
    }

    pub fn memory() -> Self {
        DbPath {
            fspath: ":memory:".to_string(),
            filename: ":memory:".to_string(),
        }
    }
}


impl<P: AsRef<std::path::Path>> From<P> for DbPath {
    fn from(p: P) -> Self {
        let fspath = p.as_ref().to_str().unwrap().to_string();
        let filename = p.as_ref().file_name().unwrap().to_str().unwrap().to_string();
        DbPath {
            fspath,
            filename,
        }
    }
}
