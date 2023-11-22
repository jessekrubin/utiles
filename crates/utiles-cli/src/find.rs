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

pub fn find_filepaths(fspaths: &[String]) -> Vec<PathBuf> {
    let fspath = fspaths[0].clone();

    let mut glob_builder = GlobSetBuilder::new();
    let glob = Glob::new("**/*.{mbtiles,sqlite,sqlite3}").unwrap();
    glob_builder.add(glob);
    let globset = glob_builder.build().unwrap();

    // filepaths
    let mut filepaths: Vec<PathBuf> = vec![];

    if is_file(&fspath) {
        filepaths.push(PathBuf::from(fspath));
    } else if is_dir(&fspath) {
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
    }
    filepaths
}
