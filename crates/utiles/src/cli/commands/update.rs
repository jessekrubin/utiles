use std::path::Path;

use tracing::{debug, warn};

use crate::cli::args::UpdateArgs;
use crate::errors::UtilesResult;
use crate::mbt::{DbChangeType, DbChangeset, MetadataChange, MetadataChangeFromTo};
use crate::sqlite::AsyncSqliteConn;
use crate::utilesqlite::mbtiles::query_distinct_tiletype_fast;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesError;

pub async fn update_mbtiles(
    filepath: &str,
    dryrun: bool,
) -> UtilesResult<MetadataChange> {
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

    let current_metadata = mbt.metadata_json().await?;

    // =========================================================
    // MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM
    // =========================================================
    let minzoom_maxzoom = mbt.query_minzoom_maxzoom().await?;
    debug!("minzoom_maxzoom: {:?}", minzoom_maxzoom);
    // updated_metadata
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
            }
        } else {
            changes.push(MetadataChangeFromTo {
                name: "minzoom".to_string(),
                from: None,
                to: Some(minzoom_maxzoom.minzoom.to_string()),
            });
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
            }
        } else {
            changes.push(MetadataChangeFromTo {
                name: "maxzoom".to_string(),
                from: None,
                to: Some(minzoom_maxzoom.maxzoom.to_string()),
            });
        }
    }

    // =====================================================================
    // FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT ~ FORMAT
    // =====================================================================
    let minmax = minzoom_maxzoom
        .ok_or(UtilesError::Error("minzoom_maxzoom is None".to_string()))?;

    // register the fn
    mbt.register_utiles_sqlite_functions().await?;
    let format = mbt.metadata_row("format").await?;
    let queryfmt = mbt
        .conn(
            // whatever clone it!
            move |c| query_distinct_tiletype_fast(c, minmax),
        )
        .await?;
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
                }
            } else {
                changes.push(MetadataChangeFromTo {
                    name: "format".to_string(),
                    from: None,
                    to: Some(fmt.clone()),
                });
            }
        }
        _ => {
            warn!("NOT IMPLEMENTED multiple formats found: {:?}", queryfmt);
        }
    }

    let metadata_change = if changes.is_empty() {
        MetadataChange::new_empty()
    } else {
        let mut updated_metadata = current_metadata.clone();
        for change in &changes {
            if let Some(new_val) = &change.to {
                updated_metadata.insert(&change.name, new_val);
            }
        }

        current_metadata.diff(&updated_metadata, true)?
    };
    if dryrun {
        warn!("Dryrun: no changes made");
    } else {
        // todo fix cloning???
        let changes2apply = vec![metadata_change.clone()];
        mbt.conn(move |conn| {
            MetadataChange::apply_changes_to_connection(conn, &changes2apply)
        })
        .await?;
    }
    Ok(metadata_change)
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
    let db_changes = DbChangeset::from(DbChangeType::from(changes));
    let jsonstring =
        serde_json::to_string_pretty(&db_changes).expect("should not fail");
    println!("{jsonstring}");
    Ok(())
}
