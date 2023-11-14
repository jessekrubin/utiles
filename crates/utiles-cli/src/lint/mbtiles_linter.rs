use std::collections::HashMap;
use std::error;
use std::path::PathBuf;
use utiles::lint_error::UtilesLintError;

pub struct MbtilesLinter {
    fspath: PathBuf,
    fix: bool,
}

const REQUIRED_METADATA_FIELDS: [&str; 7] = [
    "name", "center", "bounds", "minzoom", "maxzoom", "format", "type",
];

// iterable version
impl MbtilesLinter {
    fn new(fspath: PathBuf, fix: bool) -> MbtilesLinter {
        MbtilesLinter { fspath, fix }
    }

    pub fn lint(&self) {}
}

pub fn lint_metadata_map(map: &HashMap<String, String>) -> Vec<UtilesLintError> {
    let errs= REQUIRED_METADATA_FIELDS
        .iter()
        .filter(|key| !map.contains_key(&key.to_string()))
        .map(|key| UtilesLintError::MbtMissingMetadataKv(key.to_string()))
        .collect::<Vec<UtilesLintError>>();
    errs
    //
    // let mut errors: Vec<UtilesLintError> = Vec::new();
    // for key in REQUIRED_METADATA_FIELDS.iter() {
    //     let s = key.to_string();
    //     if !map.contains_key(&s) {
    //         errors.push(UtilesLintError::MbtMissingMetadataKv(s));
    //     }
    // }
    // return errors;
}
