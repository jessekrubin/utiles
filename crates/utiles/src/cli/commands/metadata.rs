use std::path::Path;

use serde::Serialize;
use tracing::debug;

use crate::cli::args::{MetadataArgs, MetadataSetArgs};
use crate::errors::UtilesResult;
use crate::mbt::{metadata2map, metadata2map_val, MbtilesMetadataRowParsed};
use crate::utilesqlite::Mbtiles;

pub fn metadata_main(args: &MetadataArgs) -> UtilesResult<()> {
    debug!("meta: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    assert!(
        filepath.exists(),
        "File does not exist: {}",
        filepath.display()
    );
    assert!(
        filepath.is_file(),
        "Not a file: {filepath}",
        filepath = filepath.display()
    );
    let mbtiles: Mbtiles = Mbtiles::open_existing(filepath)?;
    let metadata_rows = mbtiles.metadata()?;

    let json_val = match (args.raw, args.obj) {
        (true, true) => {
            let m = metadata2map(&metadata_rows);
            serde_json::to_value(m)?
        }
        (false, true) => {
            let values_map = metadata2map_val(&metadata_rows);
            serde_json::to_value(values_map)?
        }
        (true, false) => serde_json::to_value(metadata_rows)?,
        (false, false) => {
            let parsed_values_vec: Vec<MbtilesMetadataRowParsed> = metadata_rows
                .into_iter()
                .map(MbtilesMetadataRowParsed::from)
                .collect();
            serde_json::to_value(parsed_values_vec)?
        }
    };
    let out_str = if args.common.min {
        serde_json::to_string::<serde_json::Value>(&json_val).unwrap()
    } else {
        serde_json::to_string_pretty::<serde_json::Value>(&json_val).unwrap()
    };
    println!("{out_str}");
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct MetadataChangeFromTo {
    pub name: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub fn metadata_set_main(args: &MetadataSetArgs) -> UtilesResult<()> {
    debug!("meta: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    assert!(
        filepath.exists(),
        "File does not exist: {}",
        filepath.display()
    );
    assert!(
        filepath.is_file(),
        "Not a file: {filepath}",
        filepath = filepath.display()
    );

    let mbtiles: Mbtiles = Mbtiles::from(filepath);

    let current_value = mbtiles.metadata_get(&args.key).unwrap();
    let c = match &args.value {
        Some(value) => {
            if let Some(v) = current_value {
                if value == &v {
                    None
                } else {
                    let r = mbtiles.metadata_set(&args.key, value).unwrap();
                    debug!("metadata rows updated: {:?}", r);
                    Some(MetadataChangeFromTo {
                        name: args.key.clone(),
                        from: Some(v),
                        to: Some(value.clone()),
                    })
                }
            } else {
                let r = mbtiles.metadata_set(&args.key, value).unwrap();
                debug!("metadata rows updated: {:?}", r);
                Some(MetadataChangeFromTo {
                    name: args.key.clone(),
                    from: None,
                    to: Some(value.clone()),
                })
            }
        }
        None => {
            if current_value.is_some() {
                let r = mbtiles.metadata_delete(&args.key).unwrap();
                debug!("metadata rows deleted: {:?}", r);
                Some(MetadataChangeFromTo {
                    name: args.key.clone(),
                    from: current_value,
                    to: None,
                })
            } else {
                None
            }
        }
    };
    if let Some(c) = c {
        let str = serde_json::to_string(&c).unwrap();
        println!("{str}");
    } else {
        // print to stderr
        eprintln!("No change");
    }
    Ok(())
}
