use std::path::Path;

use tracing::{debug, warn};

use crate::cli::args::UpdateArgs;
use crate::cli::commands::metadata::MetadataChangeFromTo;
use crate::errors::UtilesResult;
use crate::utilesqlite::mbtiles::query_distinct_tiletype_fast;
use crate::utilesqlite::mbtiles_async_sqlite::AsyncSqlite;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

pub async fn update_mbtiles(
    filepath: &str,
    dryrun: bool,
) -> UtilesResult<Vec<MetadataChangeFromTo>> {
    // check that filepath exists and is file
    let mbt = if dryrun {
        MbtilesAsyncSqliteClient::open_readonly(filepath).await?
    } else {
        MbtilesAsyncSqliteClient::open_existing(filepath).await?
    };

    // check if tiles is empty...
    let tiles_is_empty = mbt.tiles_is_empty().await?;

    if tiles_is_empty {
        warn!("tiles table/view is empty: {}", filepath);
    }

    let mut changes = vec![];

    // =========================================================
    // MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM
    // =========================================================
    let minzoom_maxzoom = mbt.query_minzoom_maxzoom().await?;
    debug!("minzoom_maxzoom: {:?}", minzoom_maxzoom);

    // Updating metadata...
    let metdata_minzoom = mbt.metadata_minzoom().await?;
    if let Some(minzoom_maxzoom) = minzoom_maxzoom {
        if let Some(metadata_minzoom) = metdata_minzoom {
            if metadata_minzoom != minzoom_maxzoom.minzoom {
                changes.push(MetadataChangeFromTo {
                    name: "minzoom".to_string(),
                    from: Some(metadata_minzoom.to_string()),
                    to: Some(minzoom_maxzoom.minzoom.to_string()),
                });
                if !dryrun {
                    mbt.metadata_set("minzoom", &minzoom_maxzoom.minzoom.to_string())
                        .await?;
                }
            }
        } else {
            changes.push(MetadataChangeFromTo {
                name: "minzoom".to_string(),
                from: None,
                to: Some(minzoom_maxzoom.minzoom.to_string()),
            });
            if !dryrun {
                mbt.metadata_set("minzoom", &minzoom_maxzoom.minzoom.to_string())
                    .await?;
            }
        }
    }

    let metdata_maxzoom = mbt.metadata_maxzoom().await?;
    if let Some(minzoom_maxzoom) = minzoom_maxzoom {
        if let Some(metadata_maxzoom) = metdata_maxzoom {
            if metadata_maxzoom != minzoom_maxzoom.maxzoom {
                changes.push(MetadataChangeFromTo {
                    name: "maxzoom".to_string(),
                    from: Some(metadata_maxzoom.to_string()),
                    to: Some(minzoom_maxzoom.maxzoom.to_string()),
                });
                if !dryrun {
                    mbt.metadata_set("maxzoom", &minzoom_maxzoom.maxzoom.to_string())
                        .await?;
                }
            }
        } else {
            changes.push(MetadataChangeFromTo {
                name: "maxzoom".to_string(),
                from: None,
                to: Some(minzoom_maxzoom.maxzoom.to_string()),
            });
            if !dryrun {
                mbt.metadata_set("maxzoom", &minzoom_maxzoom.maxzoom.to_string())
                    .await?;
            }
        }
    }

    // =====================================================================
    // FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT
    // =====================================================================

    // register the fn
    mbt.register_utiles_sqlite_functions().await?;
    let format = mbt.metadata_row("format").await?;
    let queryfmt = mbt.conn(query_distinct_tiletype_fast).await?;
    match queryfmt.len() {
        0 => {
            warn!("no format found: {}", filepath);
        }
        1 => {
            let fmt = queryfmt[0].clone();
            if let Some(format) = format {
                if format.value != fmt {
                    changes.push(MetadataChangeFromTo {
                        name: "format".to_string(),
                        from: Some(format.value.clone()),
                        to: Some(fmt.clone()),
                    });
                    if !dryrun {
                        mbt.metadata_set("format", &fmt).await?;
                    }
                }
            } else {
                changes.push(MetadataChangeFromTo {
                    name: "format".to_string(),
                    from: None,
                    to: Some(fmt.clone()),
                });
                if !dryrun {
                    mbt.metadata_set("format", &fmt).await?;
                }
            }
        }
        _ => {
            warn!("NOT IMPLEMENTED multiple formats found: {:?}", queryfmt);
        }
    }
    debug!("queryfmt: {:?}", queryfmt);
    debug!("metadata changes: {:?}", changes);
    debug!("metdata_minzoom: {:?}", metdata_minzoom);
    Ok(changes)
}

pub async fn update_main(args: &UpdateArgs) -> UtilesResult<()> {
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
    let changes = update_mbtiles(&args.common.filepath, args.dryrun).await?;
    debug!("changes: {:?}", changes);
    let s = serde_json::to_string_pretty(&changes)
        .expect("should not fail; changes is a Vec<MetadataChangeFromTo>");
    println!("{s}");
    Ok(())
}
