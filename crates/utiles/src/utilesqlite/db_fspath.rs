use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbFspath {
    /// the filesystem path to the mbtiles file
    pub fspath: String,
    /// the filename of the mbtiles file (filesystem path basename)
    pub filename: String,
}

impl DbFspath {
    pub fn new(fspath: &str) -> Self {
        let p = PathBuf::from(fspath);
        let filename = p.file_name().unwrap();
        let filename = filename.to_str().unwrap().to_string();
        DbFspath {
            fspath: fspath.to_string(),
            filename,
        }
    }
}
