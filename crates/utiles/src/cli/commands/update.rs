use std::path::Path;

use tracing::{debug, info, warn};

use utiles_core::constants::MBTILES_MAGIC_NUMBER;

use crate::cli::args::UpdateArgs;
use crate::errors::UtilesResult;
use crate::mbt::mbtiles::{query_distinct_tilesize_fast, query_distinct_tiletype_fast};
use crate::mbt::{
    DbChange, DbChangeset, MetadataChange, MetadataChangeFromTo, PragmaChange,
};
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::AsyncSqliteConn;
use crate::UtilesError;

async fn update_mbt_metadata(mbt: &MbtilesClientAsync) -> UtilesResult<MetadataChange> {
    let filepath = mbt.filepath();
    // check if tiles is empty...
    let tiles_is_empty = mbt.tiles_is_empty().await?;
    if tiles_is_empty {
        info!("tiles table/view is empty: {}", filepath);
    }
    let mut metadata_changes = vec![];

    let current_metadata = mbt.metadata_json().await?;

    // =========================================================
    // MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM ~ MINZOOM ~ MAXZOOM
    // =========================================================
    let minzoom_maxzoom = mbt.query_minzoom_maxzoom().await?;
    // updated_metadata
    // Updating metadata...
    let metdata_minzoom = mbt.metadata_minzoom().await?;
    if let Some(minzoom_maxzoom) = minzoom_maxzoom {
        if let Some(metadata_minzoom) = metdata_minzoom {
            if metadata_minzoom != minzoom_maxzoom.minzoom {
                metadata_changes.push(MetadataChangeFromTo {
                    name: "minzoom".to_string(),
                    from: Some(metadata_minzoom.to_string()),
                    to: Some(minzoom_maxzoom.minzoom.to_string()),
                });
            }
        } else {
            metadata_changes.push(MetadataChangeFromTo {
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
                metadata_changes.push(MetadataChangeFromTo {
                    name: "maxzoom".to_string(),
                    from: Some(metadata_maxzoom.to_string()),
                    to: Some(minzoom_maxzoom.maxzoom.to_string()),
                });
            }
        } else {
            metadata_changes.push(MetadataChangeFromTo {
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
    let query_fmt = mbt
        .conn(
            // whatever clone it!
            move |c| query_distinct_tiletype_fast(c, minmax),
        )
        .await?;
    match query_fmt.len() {
        0 => {
            warn!("no format found: {}", filepath);
        }
        1 => {
            let fmt = query_fmt[0].clone();
            if let Some(format) = format {
                if format.value != fmt {
                    metadata_changes.push(MetadataChangeFromTo {
                        name: "format".to_string(),
                        from: Some(format.value.clone()),
                        to: Some(fmt.clone()),
                    });
                }
            } else {
                metadata_changes.push(MetadataChangeFromTo {
                    name: "format".to_string(),
                    from: None,
                    to: Some(fmt.clone()),
                });
            }
        }
        _ => {
            warn!("NOT IMPLEMENTED multiple formats found: {:?}", query_fmt);
        }
    }

    let tilesize = mbt.metadata_row("tilesize").await?;
    let query_tilesize = mbt
        .conn(
            // whatever clone it!
            move |c| query_distinct_tilesize_fast(c, minmax),
        )
        .await?;
    match query_tilesize.len() {
        0 => {
            warn!("no tilesize found: {}", filepath);
        }
        1 => {
            let ts = query_tilesize[0];
            let ts_str: String = ts.to_string();
            if let Some(tilesize) = tilesize {
                if tilesize.value != ts_str {
                    metadata_changes.push(MetadataChangeFromTo {
                        name: "tilesize".to_string(),
                        from: Some(tilesize.value.clone()),
                        to: Some(ts_str),
                    });
                }
            } else {
                metadata_changes.push(MetadataChangeFromTo {
                    name: "tilesize".to_string(),
                    from: None,
                    to: Some(ts_str),
                });
            }
        }
        _ => {
            warn!(
                "NOT IMPLEMENTED multiple tilesize found: {:?}",
                query_tilesize
            );
        }
    }

    let metadata_change = if metadata_changes.is_empty() {
        MetadataChange::new_empty()
    } else {
        let mut updated_metadata = current_metadata.clone();
        for change in &metadata_changes {
            if let Some(new_val) = &change.to {
                updated_metadata.insert(&change.name, new_val);
            }
        }

        current_metadata.diff(&updated_metadata, true)?
    };
    Ok(metadata_change)
}

pub async fn update_mbtiles_magic(
    mbt: &MbtilesClientAsync,
) -> UtilesResult<Option<PragmaChange>> {
    let magic = mbt.magic_number().await?;
    if magic == MBTILES_MAGIC_NUMBER {
        Ok(None)
    } else {
        Ok(Some(PragmaChange {
            pragma: "application_id".to_string(),
            forward: "PRAGMA application_id = 0x4d504258;".to_string(),
            reverse: format!("PRAGMA application_id = 0x{magic:x};"),
        }))
    }
}

pub async fn update_mbtiles(mbt: &MbtilesClientAsync) -> UtilesResult<Vec<DbChange>> {
    let magic_change = update_mbtiles_magic(mbt).await?;
    let mut changes = vec![];
    if let Some(magic_change) = magic_change {
        changes.push(DbChange::Pragma(magic_change));
    }
    let metadata_change = update_mbt_metadata(mbt).await?;

    if !metadata_change.is_empty() {
        changes.push(DbChange::Metadata(metadata_change));
    }

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

    // check that filepath exists and is file
    let mbt = if args.dryrun {
        MbtilesClientAsync::open_readonly(filepath).await?
    } else {
        MbtilesClientAsync::open_existing(filepath).await?
    };
    let changes = update_mbtiles(&mbt).await?;

    // if args.dryrun {
    //     warn!("Dryrun: no changes made");
    // } else {
    //     for change in changes {
    //         match change {
    //             DbChangeType::PragmaChange(pragma_change) => {
    //                 mbt.conn(move |conn| conn.execute_batch(&pragma_change.forward))
    //                     .await?;
    //             }
    //             DbChangeType::Metadata(metadata_change) => {
    //                 mbt.conn(move |conn| {
    //                     MetadataChange::apply_changes_to_connection(
    //                         conn,
    //                         // TODO: fix clone
    //                         &vec![metadata_change.clone()],
    //                     )
    //                 })
    //                     .await?;
    //             }
    //
    //             _ => {
    //                 warn!("unimplemented change: {:?}", change);
    //             }
    //         }
    //     }
    // }

    debug!("changes: {:?}", changes);
    let db_changes = DbChangeset::from_vec(changes);
    let jsonstring =
        serde_json::to_string_pretty(&db_changes).expect("should not fail");
    if args.dryrun {
        warn!("Dryrun: no changes made");
    } else {
        mbt.conn(move |conn| db_changes.apply_to_conn(conn)).await?;

        // let (sql_forward, sql_reverse) = db_changes.sql_forward_reverse();
        // info!("sql_forward:\n{}", sql_forward);
        // info!("sql_reverse:\n{}", sql_reverse);
        // mbt.conn(move |conn| conn.execute_batch(&sql_forward))
        //     .await?;
    }
    println!("{jsonstring}");
    Ok(())
}
