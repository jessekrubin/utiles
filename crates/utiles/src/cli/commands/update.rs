use std::path::Path;

use tracing::{debug, info, warn};

use crate::UtilesError;
use crate::cli::args::UpdateArgs;
use crate::errors::UtilesResult;
use crate::mbt::mbtiles::{query_distinct_tilesize_fast, query_distinct_tiletype_fast};
use crate::mbt::{
    DbChange, DbChangeset, MetadataChange, MetadataChangeFromTo, PragmaChange,
};
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::AsyncSqliteConn;
use utiles_core::MBTILES_MAGIC_NUMBER;
use utiles_core::tile_type::{TileKind, TileType};

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
    let metadata_minzoom = mbt.metadata_minzoom().await?;
    if let Some(minzoom_maxzoom) = minzoom_maxzoom {
        if let Some(metadata_minzoom) = metadata_minzoom {
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

    let metadata_maxzoom = mbt.metadata_maxzoom().await?;
    if let Some(minzoom_maxzoom) = minzoom_maxzoom {
        if let Some(metadata_maxzoom) = metadata_maxzoom {
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
    let current_kind = mbt.metadata_row("kind").await?;
    let query_fmt = mbt
        .conn(
            // whatever clone it!
            move |c| query_distinct_tiletype_fast(c, minmax),
        )
        .await?;
    let maybe_ttype = match query_fmt.len() {
        0 => {
            warn!("no format found: {}", filepath);
            None
        }
        1 => {
            let ttile = TileType::parse(query_fmt[0].as_str());

            if let Some(ttile) = ttile {
                let queried_format = ttile.format.to_string();
                let queried_kind = ttile.kind.to_string();
                if let Some(format) = format {
                    if format.value != queried_format {
                        metadata_changes.push(MetadataChangeFromTo {
                            name: "format".to_string(),
                            from: Some(format.value.clone()),
                            to: Some(queried_format.clone()),
                        });
                    }
                } else {
                    metadata_changes.push(MetadataChangeFromTo {
                        name: "format".to_string(),
                        from: None,
                        to: Some(queried_format.clone()),
                    });
                }

                if let Some(kind) = current_kind {
                    if kind.value != queried_kind {
                        metadata_changes.push(MetadataChangeFromTo {
                            name: "kind".to_string(),
                            from: Some(kind.value.clone()),
                            to: Some(queried_kind.clone()),
                        });
                    }
                } else {
                    metadata_changes.push(MetadataChangeFromTo {
                        name: "kind".to_string(),
                        from: None,
                        to: Some(queried_kind.clone()),
                    });
                }
            }
            ttile
        }
        _ => {
            warn!("NOT IMPLEMENTED multiple formats found: {:?}", query_fmt);
            None
        }
    };

    // if it is an image format check tilesize...
    if let Some(ttile) = maybe_ttype {
        if ttile.kind == TileKind::Raster {
            let tilesize = mbt.metadata_row("tilesize").await?;
            let query_tilesize = mbt
                .conn(
                    // whatever clone it!
                    move |c| query_distinct_tilesize_fast(c, minmax),
                )
                .await?;

            match query_tilesize.len() {
                0 => {
                    debug!("no tilesize found: {}", filepath);
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

pub(crate) async fn update_mbtiles_magic(
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

pub(crate) async fn update_mbtiles(
    mbt: &MbtilesClientAsync,
) -> UtilesResult<Vec<DbChange>> {
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

pub(crate) async fn update_main(args: &UpdateArgs) -> UtilesResult<()> {
    // let filepath = Path::new(&args.common.filepath);
    //
    // assert!(
    //     filepath.exists(),
    //     "File does not exist: {}",
    //     filepath.display()
    // );
    // assert!(
    //     filepath.is_file(),
    //     "Not a file: {filepath}",
    //     filepath = filepath.display()
    // );

    let filepath = Path::new(&args.common.filepath);
    // check that filepath exists and is file
    let mbt = if args.dryrun {
        MbtilesClientAsync::open_readonly(filepath).await?
    } else {
        MbtilesClientAsync::open_existing(filepath).await?
    };
    let changes = update_mbtiles(&mbt).await?;
    debug!("changes: {:?}", changes);
    let db_changes = DbChangeset::from_vec(changes);
    let jsonstring =
        serde_json::to_string_pretty(&db_changes).expect("should not fail");
    if args.dryrun {
        warn!("Dryrun: no changes made");
    } else {
        mbt.conn(move |conn| db_changes.apply_to_conn(conn)).await?;
    }
    println!("{jsonstring}");
    Ok(())
}
