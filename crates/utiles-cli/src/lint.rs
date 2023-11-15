use std::collections::HashMap;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use tracing::{debug, info, warn};
use utiles::lint_error::{UtilesLintError, UtilesLintResult};
use utiles::mbtiles::{metadata2duplicates, metadata2map};
use utilesqlite::mbtiles::{is_mbtiles, Mbtiles};

use utilesqlite::squealite;

pub const REQUIRED_METADATA_FIELDS: [&str; 7] = [
    "name", "center", "bounds", "minzoom", "maxzoom", "format", "type",
];

fn is_dir(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_dir()
}

fn is_file(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_file()
}

pub fn lint_mbtiles_file(mbtiles: Mbtiles, fix: bool) -> Vec<UtilesLintError> {
    // println!("_________ lint_filepath _________");
    // println!("lint (fix -- {fix})");
    // throw not implemented error
    // println!("{}", fspath.display());
    warn!("lint (fix -- {fix}) not implemented yet");

    // let mbtiles_result =
    //     utilesqlite::mbtiles::Mbtiles::from_filepath_str(fspath.to_str().unwrap());

    // if mbtiles_result.is_err() {
    //     println!("ERROR: {}", mbtiles_result.err().unwrap());
    //     return vec![UtilesLintError::UnableToOpen(
    //         fspath.to_str().unwrap().to_string(),
    //     )];
    // }

    let mut errors = Vec::new();
    // let mbtiles = mbtiles_result.unwrap();
    let has_unique_index_on_metadata_name =
        mbtiles.has_unique_index_on_metadata().unwrap();

    let rows = mbtiles.metadata().unwrap();

    if !has_unique_index_on_metadata_name {
        errors.push(UtilesLintError::MissingUniqueIndex(
            "metadata.name".to_string(),
        ));
    } else {
        let duplicate_rows = metadata2duplicates(rows.clone());
        if !duplicate_rows.is_empty() {
            errors.extend(
                duplicate_rows.keys().map(|k| UtilesLintError::DuplicateMetadataKey(k.clone()))
                    .collect::<Vec<UtilesLintError>>(),
            );
        }
    }
    let map = metadata2map(rows);
    let map_errs = lint_metadata_map(&map);
    if !map_errs.is_empty() {
        errors.extend(map_errs);
    }
    errors
}

pub fn lint_filepath(fspath: &Path, fix: bool) -> UtilesLintResult<Vec<UtilesLintError>> {
    let fspath_str = match fspath.to_str() {
        Some(s) => s,
        None => return Err(UtilesLintError::InvalidPath (fspath.to_str().unwrap().to_string())),
    };

    if !fspath_str.ends_with(".mbtiles") {
        let conn = match squealite::open(fspath_str) {
            Ok(conn) => conn,
            Err(e) => {
                warn!("Unable to open file: {}", e);
                return Err(UtilesLintError::UnableToOpen(fspath_str.to_string()));
            }
        };

        match is_mbtiles(&conn) {
            Ok(false) => return Ok(vec![]),
            Ok(true) => {
                let mbtiles = Mbtiles::from_conn(conn);
                return Ok(lint_mbtiles_file(mbtiles, fix));
            }
            Err(e) => {
                warn!("Unable to determine if file is mbtiles: {}", e);
                return Err(UtilesLintError::NotAMbtilesDb(fspath_str.to_string()));
            }
        }
    }

    match utilesqlite::mbtiles::Mbtiles::from_filepath_str(fspath_str) {
        Ok(mbtiles) => Ok(lint_mbtiles_file(mbtiles, fix)),
        Err(e) => {
            warn!("ERROR: {}", e);
            Err(UtilesLintError::UnableToOpen(fspath_str.to_string()))
        }
    }
}
fn lint_filepaths(fspaths: Vec<PathBuf>, fix: bool) {
    for path in fspaths {
        let r = lint_filepath(&path, fix);
        match r {
            Ok(r) => {
                debug!("r: {:?}", r);
                // print each err....
                if r.is_empty() {
                    info!("No errors found");
                } else {
                    warn!("{} - {} errors found", path.display(), r.len());

                    // let agg_err = UtilesLintError::LintErrors(r);
                    for err in r {
                        warn!("{}", err.to_string());
                    }
                }
            }
            Err(e) => {
                warn!("Unable to open file: {}", e);
                warn!("Error: {}", e);
            }
        }
        // let r = lint_mbtiles_file(&path, fix);
        // debug!("r: {:?}", r);
        // // print each err....
        // if r.is_empty() {
        //     info!("No errors found");
        // } else {
        //     warn!("{} - {} errors found", path.display(), r.len());
        //
        //     // let agg_err = UtilesLintError::LintErrors(r);
        //     for err in r {
        //         warn!("{}", err.to_string());
        //     }
        // }
    }
}

pub fn find_filepaths(fspaths: Vec<String>) -> Vec<PathBuf> {
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
        for result in walk_builder.build().filter_map(|e| e.ok()) {
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

pub fn lint_main(fspaths: Vec<String>, fix: bool) {
    let filepaths = find_filepaths(fspaths);
    if fix {
        warn!("lint fix is not implemented yet");
    }
    debug!("filepaths: {:?}", filepaths);
    if filepaths.is_empty() {
        warn!("No files found");
        return;
    }

    lint_filepaths(filepaths, fix)
}

pub fn lint_metadata_map(map: &HashMap<String, String>) -> Vec<UtilesLintError> {
    let errs = REQUIRED_METADATA_FIELDS
        .iter()
        .filter(|key| !map.contains_key(&key.to_string()))
        .map(|key| UtilesLintError::MbtMissingMetadataKv(key.to_string()))
        .collect::<Vec<UtilesLintError>>();
    errs
}