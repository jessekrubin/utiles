use std::path::Path;

use serde::Serialize;
use tracing::debug;

use utiles_core::mbutiles::metadata2map;

use crate::cli::args::{MetadataArgs, MetadataSetArgs};
use crate::utilesqlite::Mbtiles;

pub fn metadata_main(args: &MetadataArgs) {
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
    let metadata_rows = mbtiles.metadata().unwrap();

    let json_val = if args.obj {
        let m = metadata2map(&metadata_rows);
        serde_json::to_value(m).unwrap()

        // serde_json::to_value(&metadata_rows).unwrap()
    } else {
        serde_json::to_value(&metadata_rows).unwrap()
    };

    let out_str = if args.common.min {
        serde_json::to_string::<serde_json::Value>(&json_val).unwrap()
    } else {
        serde_json::to_string_pretty::<serde_json::Value>(&json_val).unwrap()
    };
    println!("{out_str}");
}

#[derive(Debug, Serialize)]
struct MetadataChangeFromTo {
    pub name: String,
    pub from: Option<String>,
    pub to: Option<String>,
}
//
// pub enum MetadataDiffType {
//     Insert,
//     Update,
//     Delete,
// }
// #[derive(Debug, Serialize)]
// struct MetadataInsert {
//     #[serde(rename = "type")]
//     pub type_: MetadataDiffType,
//     pub name: String,
//     pub value: String,
// }
//
// #[derive(Debug, Serialize)]
// struct MetadataChange {
//     pub name: String,
//     pub value: Option<String>,
// }

pub fn metadata_set_main(args: &MetadataSetArgs) {
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
                if value != &v {
                    let r = mbtiles.metadata_set(&args.key, value).unwrap();
                    debug!("metadata rows updated: {:?}", r);
                    Some(MetadataChangeFromTo {
                        name: args.key.clone(),
                        from: Some(v),
                        to: Some(value.clone()),
                    })
                } else {
                    None
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
            // match current_value {
            //     Some(v) => {
            //         if value != &v {
            //             let r = mbtiles.metadata_set(&args.key, value).unwrap();
            //             debug!("metadata rows updated: {:?}", r);
            //             Some(MetadataChangeFromTo {
            //                 name: args.key.clone(),
            //                 from: Some(v),
            //                 to: Some(value.clone()),
            //             })
            //         } else {
            //             None
            //         }
            //     }
            //     None => {
            //         let r = mbtiles.metadata_set(&args.key, value).unwrap();
            //         debug!("metadata rows updated: {:?}", r);
            //         Some(MetadataChangeFromTo {
            //             name: args.key.clone(),
            //             from: None,
            //             to: Some(value.clone()),
            //         })
            //     }
            // }
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
}
