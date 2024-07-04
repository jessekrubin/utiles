use std::path::Path;

use crate::cli::args::{MetadataArgs, MetadataSetArgs};
use crate::cli::stdin2string::stdin2string;
use crate::errors::UtilesResult;
use crate::mbt::{
    metadata2map, metadata2map_val, read_metadata_json, MbtilesMetadataJson,
    MbtilesMetadataRowParsed, MetadataChange,
};
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use serde::Serialize;
use tracing::warn;
use tracing::{debug, info};

pub async fn metadata_main(args: &MetadataArgs) -> UtilesResult<()> {
    debug!("meta: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    let mbtiles = MbtilesAsyncSqliteClient::open_existing(filepath).await?;
    let metadata_rows = mbtiles.metadata_rows().await?;

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
                .map(|row| MbtilesMetadataRowParsed::from(&row))
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

pub async fn metadata_set_main(args: &MetadataSetArgs) -> UtilesResult<()> {
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
    let current_metadata_json = mbtiles.metadata_json()?;
    let c = match &args.value {
        Some(value) => {
            let mut mdjson = current_metadata_json.clone();

            println!("BOOM");
            mdjson.insert(&args.key, value);

            let (forward, inverse, data) =
                current_metadata_json.diff(&mdjson, false)?;

            MetadataChange::from_forward_reverse_data(forward, inverse, data)
        }
        None => {
            // check if key is filepath ending in .json then load and
            // update from there!

            if args.key.to_lowercase().ends_with(".json") {
                // get metadata from json file...

                let mdjson = read_metadata_json(&args.key).await?;
                let (forward, inverse, data) =
                    current_metadata_json.diff(&mdjson, true)?;

                MetadataChange::from_forward_reverse_data(forward, inverse, data)
            } else if args.key.to_lowercase() == "-" || args.key.to_lowercase() == "--"
            {
                // get metadata from stdin...
                let stdin_str = stdin2string()?;
                let mdjson = serde_json::from_str::<MbtilesMetadataJson>(&stdin_str)?;
                let (forward, inverse, data) =
                    current_metadata_json.diff(&mdjson, true)?;

                MetadataChange::from_forward_reverse_data(forward, inverse, data)
            } else {
                let mut mdjson = current_metadata_json.clone();
                mdjson.delete(&args.key);

                let (forward, inverse, data) =
                    current_metadata_json.diff(&mdjson, false)?;

                MetadataChange::from_forward_reverse_data(forward, inverse, data)
            }
        }
    };

    if c.is_empty() {
        info!("No change");
    } else {
        debug!("metadata change: {c:?}");
        let stringy = serde_json::to_string_pretty(&c)?;
        println!("{stringy}");
        if args.dryrun {
            warn!("Dryrun: no changes made");
        } else {
            MetadataChange::apply_changes_to_connection(&mbtiles.conn, &vec![c])?;
        }
    }
    Ok(())
}

//
// pub fn metadata_set_main_og(args: &MetadataSetArgs) -> UtilesResult<()> {
//     debug!("meta: {}", args.common.filepath);
//     // check that filepath exists and is file
//     let filepath = Path::new(&args.common.filepath);
//     assert!(
//         filepath.exists(),
//         "File does not exist: {}",
//         filepath.display()
//     );
//     assert!(
//         filepath.is_file(),
//         "Not a file: {filepath}",
//         filepath = filepath.display()
//     );
//
//     let mbtiles: Mbtiles = Mbtiles::from(filepath);
//
//     let current_value = mbtiles.metadata_get(&args.key).unwrap();
//     let c = match &args.value {
//         Some(value) => {
//             if let Some(v) = current_value {
//                 if value == &v {
//                     None
//                 } else {
//                     let r = mbtiles.metadata_set(&args.key, value).unwrap();
//                     debug!("metadata rows updated: {:?}", r);
//                     Some(MetadataChangeFromTo {
//                         name: args.key.clone(),
//                         from: Some(v),
//                         to: Some(value.clone()),
//                     })
//                 }
//             } else {
//                 let r = mbtiles.metadata_set(&args.key, value).unwrap();
//                 debug!("metadata rows updated: {:?}", r);
//                 Some(MetadataChangeFromTo {
//                     name: args.key.clone(),
//                     from: None,
//                     to: Some(value.clone()),
//                 })
//             }
//         }
//         None => {
//             // check if key is filepath ending in .json then load and
//             // update from there!
//
//
//             // if args.key.ends_with(".json") {
//             //     let json_str = std::fs::read_to_string(&args.key).unwrap();
//             //     let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
//             //     let r = mbtiles.metadata_set_json(&args.key, &json).unwrap();
//             //     debug!("metadata rows updated: {:?}", r);
//             //     Some(MetadataChangeFromTo {
//             //         name: args.key.clone(),
//             //         from: current_value,
//             //         to: Some(json_str),
//             //     })
//             // } else
//
//             if current_value.is_some() {
//                 let r = mbtiles.metadata_delete(&args.key).unwrap();
//                 debug!("metadata rows deleted: {:?}", r);
//                 Some(MetadataChangeFromTo {
//                     name: args.key.clone(),
//                     from: current_value,
//                     to: None,
//                 })
//             } else {
//                 None
//             }
//         }
//     };
//     if let Some(c) = c {
//         let str = serde_json::to_string(&c).unwrap();
//         println!("{str}");
//     } else {
//         // print to stderr
//         eprintln!("No change");
//     }
//     Ok(())
// }
