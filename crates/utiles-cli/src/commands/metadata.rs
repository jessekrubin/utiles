use std::path::Path;

use serde::Serialize;
use tracing::debug;

use utiles::mbtiles::metadata2map;
use utilesqlite::Mbtiles;

use crate::args::{MetadataArgs, MetadataSetArgs};

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
    if let Some(v) = current_value {
        if args.value != v {
            let r = mbtiles.metadata_set(&args.key, &args.value).unwrap();
            debug!("metadata rows updated: {:?}", r);
        }
    } else {
        let r = mbtiles.metadata_set(&args.key, &args.value).unwrap();
        debug!("metadata rows updated: {:?}", r);
    }
    // match current_value {
    //     Some(v) => {
    //         if args.value != v {
    //             let r = mbtiles.metadata_set(&args.key, &args.value).unwrap();
    //             debug!("metadata rows updated: {:?}", r);
    //         }
    //     }
    //     None => {
    //         let r = mbtiles.metadata_set(&args.key, &args.value).unwrap();
    //         debug!("metadata rows updated: {:?}", r);
    //     }
    // }

    let c = MetadataChangeFromTo {
        name: args.key.clone(),
        from: None,
        to: None,
    };
    let str = serde_json::to_string(&c).unwrap();
    println!("{str}");
}
