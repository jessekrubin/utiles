use std::path::PathBuf;

use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use tracing::warn;

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
    let glob = Glob::new("**/*.{mbtiles,sqlite,sqlite3}").unwrap();
    glob_builder.add(glob);
    let globset = glob_builder.build().unwrap();

    let dirpath = PathBuf::from(fspath).canonicalize().unwrap();
    let walk_builder = WalkBuilder::new(dirpath);
    for result in walk_builder.build().filter_map(std::result::Result::ok) {
        if !result.file_type().unwrap().is_file() {
            continue;
        }
        match result.path().to_str() {
            Some(path) => {
                if globset.is_match(path) {
                    filepaths.push(path.into());
                }
            }
            None => {
                warn!("Unable to convert path to string: {:?}", result.path());
            }
        }
    }
    filepaths
}

pub fn find_filepaths(fspaths: &[String]) -> Vec<PathBuf> {
    // split the paths up into files and dirs/patterns
    let mut files: Vec<String> = vec![];
    let mut dirs: Vec<String> = vec![];
    for fspath in fspaths {
        if is_file(fspath) {
            files.push(fspath.clone());
        } else if is_dir(fspath) {
            dirs.push(fspath.clone());
        } else {
            warn!("{} is not a file or directory", fspath);
        }
    }

    let mut filepaths: Vec<PathBuf> = vec![];
    for fspath in files {
        filepaths.extend(find_datasets(&fspath));
    }

    for fspath in dirs {
        filepaths.extend(find_datasets(&fspath));
    }
    filepaths
}
