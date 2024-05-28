use std::path::PathBuf;

use crate::errors::UtilesResult;
use crate::UtilesError;
use globset::{Glob, GlobSetBuilder};
use tracing::{debug, warn};
use walkdir::WalkDir;

fn is_dir(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_dir()
}

fn is_file(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_file()
}

pub fn find_datasets(fspath: &str) -> UtilesResult<Vec<PathBuf>> {
    // filepaths
    let mut filepaths: Vec<PathBuf> = vec![];
    let mut glob_builder = GlobSetBuilder::new();
    let glob_recursive = Glob::new("**/*.{mbtiles,sqlite,sqlite3}")
        .map_err(UtilesError::GlobsetError)?;
    glob_builder.add(glob_recursive);
    let glob =
        Glob::new("*.{mbtiles,sqlite,sqlite3}").map_err(UtilesError::GlobsetError)?;
    glob_builder.add(glob);

    let globset = glob_builder.build()?;
    for entry in WalkDir::new(fspath).into_iter().flatten() {
        // filter_map(std::result::Result::ok)
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(path) = path.to_str() {
                if globset.is_match(path) {
                    filepaths.push(path.into());
                }
            }
        }
    }
    Ok(filepaths)
}

pub fn find_filepaths(fspaths: &[String]) -> UtilesResult<Vec<PathBuf>> {
    // split the paths up into files and dirs/patterns
    let mut files: Vec<String> = vec![];
    let mut dirs: Vec<String> = vec![];
    debug!("searching fspaths: {:?}", fspaths);
    for fspath in fspaths {
        debug!("fspath: {}", fspath);
        if fspath == "." {
            // get the current working directory and resolve it to an absolute path
            let cwd =
                std::env::current_dir().map_err(|e| UtilesError::Str(e.to_string()))?;
            let cwd_to_str = cwd.to_str().ok_or(UtilesError::Str("cwd".to_string()))?;
            dirs.push(cwd_to_str.to_string());
        } else if is_file(fspath) {
            files.push(fspath.clone());
        } else if is_dir(fspath) {
            dirs.push(fspath.clone());
        } else {
            warn!("{} is not a file or directory", fspath);
        }
    }
    debug!("files: {:?}", files);
    debug!("dirs: {:?}", dirs);

    let mut filepaths: Vec<PathBuf> = vec![];
    for fspath in files {
        let paths = find_datasets(&fspath)?;
        filepaths.extend(paths);
    }
    for fspath in dirs {
        let paths = find_datasets(&fspath)?;
        filepaths.extend(paths);
    }
    Ok(filepaths)
}
