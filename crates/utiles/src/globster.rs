use std::path::PathBuf;

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

pub fn find_datasets(fspath: &str) -> Vec<PathBuf> {
    // filepaths
    let mut filepaths: Vec<PathBuf> = vec![];
    let mut glob_builder = GlobSetBuilder::new();
    let glob_recursive = Glob::new("**/*.{mbtiles,sqlite,sqlite3}").unwrap();
    glob_builder.add(glob_recursive);
    let glob = Glob::new("*.{mbtiles,sqlite,sqlite3}").unwrap();
    glob_builder.add(glob);

    let globset = glob_builder.build().unwrap();
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
    filepaths
}

pub fn find_filepaths(fspaths: &[String]) -> Vec<PathBuf> {
    // split the paths up into files and dirs/patterns
    let mut files: Vec<String> = vec![];
    let mut dirs: Vec<String> = vec![];
    debug!("searching fspaths: {:?}", fspaths);
    for fspath in fspaths {
        debug!("fspath: {}", fspath);
        if fspath == "." {
            // get the current working directory and resolve it to an absolute path
            let pwd = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            dirs.push(pwd);
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
        filepaths.extend(find_datasets(&fspath));
    }

    for fspath in dirs {
        filepaths.extend(find_datasets(&fspath));
    }
    filepaths
}
