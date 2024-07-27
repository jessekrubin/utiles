use std::path::Path;

use tracing::warn;
use tracing::{debug, info};

use crate::cli::args::{MetadataArgs, MetadataSetArgs};
use crate::cli::stdin2string::stdin2string;
use crate::errors::UtilesResult;
use crate::fs_async::file_exists_err;
use crate::mbt::{
    metadata2map, metadata2map_val, read_metadata_json, DbChange, DbChangeset,
    MbtilesMetadataJson, MbtilesMetadataRowParsed,
};
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::AsyncSqliteConn;

pub async fn metadata_main(args: &MetadataArgs) -> UtilesResult<()> {
    debug!("meta: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    let mbtiles = MbtilesClientAsync::open_existing(filepath).await?;
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
        serde_json::to_string::<serde_json::Value>(&json_val)
    } else {
        serde_json::to_string_pretty::<serde_json::Value>(&json_val)
    }?;

    println!("{out_str}");
    Ok(())
}

pub async fn metadata_set_main(args: &MetadataSetArgs) -> UtilesResult<()> {
    debug!("meta: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    file_exists_err(filepath).await?;
    let mbtiles = MbtilesClientAsync::open_existing(filepath).await?;
    let current_metadata_json = mbtiles.metadata_json().await?;
    let c = match &args.value {
        Some(value) => {
            let mut mdjson = current_metadata_json.clone();
            mdjson.insert(&args.key, value);
            current_metadata_json.diff(&mdjson, true)?
        }
        None => {
            // check if key is filepath ending in .json then load and
            // update from there!

            if args.key.to_lowercase().ends_with(".json") {
                // get metadata from json file...

                let mdjson = read_metadata_json(&args.key).await?;
                debug!("mdjson: {:?}", mdjson);
                current_metadata_json.diff(&mdjson, true)?
            } else if args.key.to_lowercase() == "-" || args.key.to_lowercase() == "--"
            {
                // get metadata from stdin...
                let stdin_str = stdin2string()?;
                let mdjson = serde_json::from_str::<MbtilesMetadataJson>(&stdin_str)?;
                current_metadata_json.diff(&mdjson, true)?
            } else {
                let mut mdjson = current_metadata_json.clone();
                mdjson.delete(&args.key);
                current_metadata_json.diff(&mdjson, true)?
            }
        }
    };
    debug!("metadata change: {:?}", c);
    let db_change = DbChangeset::from_vec(vec![DbChange::Metadata(c)]);
    if db_change.is_empty() {
        info!("No changes to make");
    } else {
        let json_str = serde_json::to_string_pretty(&db_change)?;
        println!("{json_str}");
        if args.dryrun {
            warn!("Dryrun: no changes made");
        } else {
            mbtiles
                .conn(move |conn| db_change.apply_to_conn(conn))
                .await?;
        }
    }
    Ok(())
}
