use std::collections::HashMap;
use std::path::PathBuf;

use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use utiles::lint_error::UtilesLintResult;
use utiles::mbtiles::metadata_row::{MbtilesMetadataRow, MbtilesMetadataRows};

fn is_dir(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_dir()
}

fn is_file(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_file()
}

struct MbtilesLinter {
    fspath: PathBuf,
    fix: bool,
}

// iterable version
impl MbtilesLinter {
    fn new(fspath: PathBuf, fix: bool) -> MbtilesLinter {
        MbtilesLinter { fspath, fix }
    }
}

fn check_metadata_for_duplicates(
    rows: Vec<MbtilesMetadataRow>,
) -> HashMap<String, Vec<MbtilesMetadataRow>> {
    let mut map: HashMap<String, Vec<MbtilesMetadataRow>> = HashMap::new();
    for row in rows {
        map.entry(row.name.clone()).or_insert(Vec::new()).push(row);
    }

    // filter out the non-duplicates
    map.into_iter().filter(|(_k, v)| v.len() > 1).collect()
}

fn metadata2map(
    rows: MbtilesMetadataRows,
) -> UtilesLintResult<HashMap<String, String>> {
    let mut map: HashMap<String, String> = HashMap::from_iter(
        rows.iter().map(|row| (row.name.clone(), row.value.clone())),
    );
    Ok(map)
}

fn lint_mbtiles_file(fspath: PathBuf, fix: bool) {
    println!("_________ lint_filepath _________");
    println!("lint (fix -- {fix})");
    // throw not implemented error
    println!("{}", fspath.display());

    let mbtiles_result =
        utilesqlite::mbtiles::Mbtiles::from_filepath_str(fspath.to_str().unwrap());
    if mbtiles_result.is_err() {
        println!("ERROR: {}", mbtiles_result.err().unwrap());
        return;
    }

    let mbtiles = mbtiles_result.unwrap();
    let has_unique_index_on_metadata_name =
        mbtiles.has_unique_index_on_metadata().unwrap();

    if !has_unique_index_on_metadata_name {
        println!("ERROR: metadata.name does not have a unique index");
        // return;
    }
    let rows = mbtiles.metadata().unwrap();
    let duplicate_rows = check_metadata_for_duplicates(rows.clone());
    if duplicate_rows.len() > 0 {
        println!("ERROR: metadata.name has duplicate values");
        // return;
    }

    let map = metadata2map(rows).unwrap();

    // has name
    if !map.contains_key("name") {
        println!("ERROR: metadata does not have a name");
        return;
    }

    // has center
    if !map.contains_key("center") {
        println!("ERROR: metadata does not have a center");
        return;
    }

    // has bounds
    if !map.contains_key("bounds") {
        println!("ERROR: metadata does not have a bounds");
        return;
    } else {
        let bounds_value = map.get("bounds").unwrap();
        println!("bounds_value: {}", bounds_value);
        if bounds_value.is_empty() {
            println!("ERROR: metadata.bounds is empty");
            return;
        }
        //     parsed
        let bounds = utiles::bbox::BBox::from(bounds_value);
    }

    // has minzoom
    if !map.contains_key("minzoom") {
        println!("ERROR: metadata does not have a minzoom");
        return;
    }

    // has maxzoom
    if !map.contains_key("maxzoom") {
        println!("ERROR: metadata does not have a maxzoom");
        return;
    }

    // has name
    if !map.contains_key("name") {
        println!("ERROR: metadata does not have a name");
        return;
    } else {
        let name_value = map.get("name").unwrap();
        if name_value.is_empty() {
            println!("ERROR: metadata.name is empty");
            return;
        }
    }

    // has format
    if !map.contains_key("format") {
        println!("ERROR: metadata does not have a format");
        return;
    } else {
        let format_value = map.get("format").unwrap();
        // allowd formats are png, jpg, pbf, webp, geojson
        if format_value != "png"
            && format_value != "jpg"
            && format_value != "pbf"
            && format_value != "webp"
            && format_value != "geojson"
        {
            println!("ERROR: metadata.format is not 'png', 'jpg', 'pbf', 'webp', or 'geojson'");
            return;
        }
    }

    // has type
    if !map.contains_key("type") {
        println!("ERROR: metadata does not have a type");
        return;
    } else {
        let type_value = map.get("type").unwrap();
        if type_value != "overlay" && type_value != "baselayer" {
            println!("ERROR: metadata.type is not 'overlay' or 'baselayer'");
            return;
        }
    }
}

fn lint_filepaths(fspaths: Vec<PathBuf>, fix: bool) {
    println!("lint (fix -- {fix})");
    // throw not implemented error
    for path in fspaths {
        lint_mbtiles_file(path, fix);
    }
}

pub fn lint_main(fspath: String, fix: bool) {
    println!("lint (fix -- {fix}): {fspath}");

    if is_file(&fspath) {
        println!("is file");
        return lint_filepaths(vec![PathBuf::from(fspath)], fix);
    }
    // throw not implemented error
    if is_dir(&fspath) {
        println!("is dir");
        // replace \ with /
        let dirpath = PathBuf::from(fspath).canonicalize().unwrap();
        println!("{}", dirpath.display());
        let mut glob_builder = GlobSetBuilder::new();
        let glob = Glob::new("**/*.{mbtiles,sqlite,sqlite3}").unwrap();
        glob_builder.add(glob);
        let globset = glob_builder.build().unwrap();

        // replace \ with /

        // dirpath + glob
        let glob_pattern = dirpath.join("**/*.{mbtiles,sqlite,sqlite3}");
        let mut filepaths: Vec<PathBuf> = Vec::new();
        let walk_builder = WalkBuilder::new(dirpath);
        for result in walk_builder.build().into_iter().filter_map(|e| e.ok()) {
            // skip non-files
            if !result.file_type().unwrap().is_file() {
                continue;
            }
            match result.path().to_str() {
                Some(path) => {
                    if globset.is_match(path) {
                        filepaths.push(path.into());
                        println!("{}", path);
                    }
                }
                None => {
                    println!("ERROR: path is not valid UTF-8");
                }
            }
        }

        lint_filepaths(filepaths, fix);
        // ) {
        //     match result {
        //         Ok(entry) => {
        //             println!("{}", entry.path().display());
        //         }
        //         Err(err) => {
        //             println!("ERROR: {}", err);
        //         }
        //     }
        // }
    }
}
