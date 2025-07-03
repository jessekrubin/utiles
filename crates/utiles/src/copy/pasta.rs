use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use utiles_core::UtilesCoreError;

use crate::UtilesError;
use crate::copy::CopyConfig;
use crate::errors::UtilesCopyError;
use crate::errors::UtilesResult;
use crate::hash::xxh64_be_hex_upper;
use crate::mbt::{
    MbtStreamWriterSync, MbtType, MbtWriterStats, Mbtiles, MbtilesMetadataJson,
};
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::{AsyncSqliteConn, Sqlike3Async};

#[derive(Debug)]
pub(super) struct CopyPasta {
    pub cfg: CopyConfig,
}

#[derive(Debug)]
pub(super) struct CopyPastaPreflightAnalysis {
    pub dst_db_type: MbtType,
    pub dst_db: MbtilesClientAsync,

    pub src_db_type: MbtType,
    pub src_db: MbtilesClientAsync,
    pub src_db_metadata: Option<MbtilesMetadataJson>,

    pub dst_is_new: bool,
    pub check_conflict: bool,
}

impl CopyPasta {
    pub(super) fn new(cfg: CopyConfig) -> UtilesResult<CopyPasta> {
        cfg.check()?;
        // sanity check stuff here...
        Ok(Self { cfg })
    }

    pub(super) async fn get_src_db(&self) -> UtilesResult<MbtilesClientAsync> {
        // do the thing
        let src_db = MbtilesClientAsync::open_existing(&self.cfg.src).await?;
        debug!("src_db: {:?}", src_db);
        Ok(src_db)
    }

    /// Returns the destination db and a bool indicating if it was created
    pub(super) async fn get_dst_db(
        &self,
        dst_db_type: Option<MbtType>,
    ) -> UtilesResult<(MbtilesClientAsync, bool, MbtType)> {
        // if the dst is a file... we gotta get it...
        let dst_db_res = MbtilesClientAsync::open_existing(&self.cfg.dst).await;
        let (dst_db, is_new) = match dst_db_res {
            Ok(db) => (db, false),
            Err(e) => {
                debug!("dst_db_res: {:?}", e);
                debug!("Creating new db... {:?}", self.cfg.dst);
                // type is
                debug!("dbtype: {:?}", self.cfg.dst_type);
                let db =
                    MbtilesClientAsync::open_new(&self.cfg.dst, dst_db_type).await?;
                (db, true)
            }
        };
        dst_db.register_utiles_sqlite_functions().await?;
        let db_type_queried = dst_db.query_mbt_type().await?;
        Ok((dst_db, is_new, db_type_queried))
    }

    pub(super) async fn set_metadata(
        &self,
        dst_db: &MbtilesClientAsync,
        metadata: MbtilesMetadataJson,
    ) -> UtilesResult<usize> {
        let metadata_rows = metadata.as_obj();
        // if we have any bboxes... should set them...
        let mut n_metadata_inserted = 0;
        for row in metadata_rows {
            let (name, value) = row;
            let value_string = if let serde_json::Value::String(s) = value {
                s
            } else {
                serde_json::to_string(&value)?
            };
            debug!("metadata: {:?} -> {:?}", name, value_string);
            let res = dst_db.metadata_set(&name, &value_string).await?;
            n_metadata_inserted += res;
        }
        Ok(n_metadata_inserted)
    }
    // pub async fn copy_metadata(
    //     &self,
    //     dst_db: &MbtilesClientAsync,
    // ) -> UtilesResult<usize> {
    //     let src_db = self.get_src_db().await?;
    //     let metadata_rows = src_db.metadata_json().await?;
    //     self.set_metadata(dst_db, metadata_rows).await
    // }

    pub(super) async fn copy_tiles_zbox_flat(
        &self,
        dst_db: &MbtilesClientAsync,
    ) -> UtilesResult<usize> {
        let src_db_name = "src";
        let where_clause = self.cfg.mbtiles_sql_where()?;
        let insert_strat = self.cfg.istrat.sql_prefix().to_string();

        let n_tiles_inserted = dst_db.conn(
            move |x| {
                let insert_statement = &format!(
                    "{insert_strat} INTO tiles (zoom_level, tile_column, tile_row, tile_data) SELECT zoom_level, tile_column, tile_row, tile_data FROM {src_db_name}.tiles {where_clause}"
                );
                debug!("Executing tiles insert: {:?}", insert_statement);

                x.execute(
                    insert_statement,
                    [],
                )
            }
        ).await?;

        if n_tiles_inserted == 0 {
            warn!("No tiles inserted!");
        } else {
            debug!("n_tiles_inserted: {:?}", n_tiles_inserted);
        }
        Ok(n_tiles_inserted)
    }

    pub(super) async fn copy_tiles_zbox_hash(
        &self,
        dst_db: &MbtilesClientAsync,
    ) -> UtilesResult<usize> {
        let src_db_name = "src";
        let where_clause = self.cfg.mbtiles_sql_where()?;
        let insert_strat = self.cfg.istrat.sql_prefix().to_string();
        let hash_type_fn_string =
            self.cfg.hash.unwrap_or_default().sqlite_hex_fn_name();
        let n_tiles_inserted = dst_db.conn(
            move |x| {
                let insert_statement = &format!(
                    "{insert_strat} INTO tiles_with_hash (zoom_level, tile_column, tile_row, tile_data, tile_hash) SELECT zoom_level, tile_column, tile_row, tile_data, {hash_type_fn_string}(tile_data) as tile_hash FROM {src_db_name}.tiles {where_clause}"
                );
                debug!("Executing tiles insert: {:?}", insert_statement);
                x.execute(
                    insert_statement,
                    [],
                )
            }
        ).await?;

        if n_tiles_inserted == 0 {
            warn!("No tiles inserted!");
        } else {
            debug!("n_tiles_inserted: {:?}", n_tiles_inserted);
        }
        Ok(n_tiles_inserted)
    }

    pub(super) async fn copy_tiles_zbox_norm(
        &self,
        dst_db: &MbtilesClientAsync,
    ) -> UtilesResult<usize> {
        let src_db_name = "src";
        let where_clause = self.cfg.mbtiles_sql_where()?;
        let insert_strat = self.cfg.istrat.sql_prefix().to_string();
        let hash_type_fn_string =
            self.cfg.hash.unwrap_or_default().sqlite_hex_fn_name();
        debug!("hash fn: {}", hash_type_fn_string);

        let n_tiles_inserted = dst_db
            .conn(move |x| {
                let insert_statement = &format!(
                    "
{insert_strat} INTO map (zoom_level, tile_column, tile_row, tile_id)
SELECT
    zoom_level as zoom_level,
    tile_column as tile_column,
    tile_row as tile_row,
    {hash_type_fn_string}(tile_data) AS tile_id
FROM
    {src_db_name}.tiles
{where_clause};
"
                );
                debug!("Executing tiles insert: {:?}", insert_statement);
                let changes = x.execute(insert_statement, [])?;

                // now just join and insert the images...
                let insert_statement = &format!(
                    "
INSERT OR IGNORE INTO images (tile_id, tile_data)
SELECT
    map.tile_id,
    tiles.tile_data
FROM
    map
JOIN
    {src_db_name}.tiles
ON
    map.zoom_level = {src_db_name}.tiles.zoom_level
    AND map.tile_column = {src_db_name}.tiles.tile_column
    AND map.tile_row = {src_db_name}.tiles.tile_row;
                    "
                );
                debug!("Executing images insert: {:?}", insert_statement);
                let changes2 = x.execute(insert_statement, [])?;
                Ok(changes + changes2)
            })
            .await?;

        if n_tiles_inserted == 0 {
            warn!("No tiles inserted!");
        } else {
            debug!("n_tiles_inserted: {:?}", n_tiles_inserted);
        }
        Ok(n_tiles_inserted)
    }

    pub(super) async fn copy_tiles_with_attach(
        &self,
        dst_db: &MbtilesClientAsync,
    ) -> UtilesResult<usize> {
        debug!("copy tiles zbox");

        // TODO: check the dst type else where
        let dst_db_type = dst_db.query_mbt_type().await?;
        debug!("dst_db_type: {:?}", dst_db_type);
        let res = match dst_db_type {
            MbtType::Flat => {
                // do the thing
                debug!("Copying tiles from src to dst: {:?}", self.cfg);
                self.copy_tiles_zbox_flat(dst_db).await
            }
            MbtType::Hash => {
                // do the thing
                debug!("Copying tiles from src to dst: {:?}", self.cfg);
                self.copy_tiles_zbox_hash(dst_db).await
            }
            MbtType::Norm => {
                // do the thing
                debug!("Copying tiles from src to dst: {:?}", self.cfg);
                self.copy_tiles_zbox_norm(dst_db).await
            }
            _ => {
                // do the thing
                debug!("Copying tiles from src to dst: {:?}", self.cfg);
                let emsg = format!("Unsupported/unimplemented db-type {dst_db_type:?}");
                Err(UtilesCoreError::Unimplemented(emsg).into())
            }
        }?;
        debug!("res: {:?}", res);
        Ok(res)
    }

    pub(super) async fn preflight_check(
        &self,
    ) -> UtilesResult<CopyPastaPreflightAnalysis> {
        // do the thing
        debug!("Preflight check: {:?}", self.cfg);
        let src_db = self.get_src_db().await?;
        let src_db_type = if (src_db.mbtype == MbtType::Planetiler
            || src_db.mbtype == MbtType::Tippecanoe)
            && (self.cfg.dst_type.is_none())
        {
            let msg = format!(
                "No dst-type provided and src-type is {} which is an unimplemented dst-type",
                src_db.mbtype
            );
            warn!("{msg}");
            MbtType::Norm
        } else {
            src_db.mbtype
        };
        let src_db_metadata = if let Ok(m) = src_db.metadata_json().await {
            Some(m)
        } else {
            debug!("Error getting metadata from src db");
            None
        };

        let dst_db_type_if_new = self.cfg.dst_type.or(Some(src_db_type));
        info!("dst_db_type_if_new: {:?}", dst_db_type_if_new);

        // if dst exists... get it and type...
        let (dst_db, is_new, db_type) = self.get_dst_db(dst_db_type_if_new).await?;
        Ok(CopyPastaPreflightAnalysis {
            src_db,
            src_db_type,
            src_db_metadata,
            dst_db,
            dst_db_type: db_type,
            dst_is_new: is_new,
            check_conflict: self.cfg.istrat.requires_check() && !is_new,
        })
    }

    pub(super) async fn check_conflict(
        &self,
        dst_db: &MbtilesClientAsync,
    ) -> UtilesResult<bool> {
        // do the thing
        debug!("Check overlapping: {:?}", self.cfg);

        // join on zoom_level, tile_column, tile_row for src and dst and
        // see if there is any overlap

        let where_clause = self.cfg.mbtiles_sql_where()?;
        // TODO: check if minzoom and maxzoom overlap
        let has_conflict = dst_db
            .conn(move |c| {
                let src_db_name = "src";
                let check_statement = &format!(
                    r"
SELECT COUNT(*)
FROM (
    SELECT main.tiles.zoom_level, main.tiles.tile_column, main.tiles.tile_row
    FROM main.tiles
    {where_clause}
) AS filtered_tiles
JOIN {src_db_name}.tiles ON
    filtered_tiles.zoom_level = {src_db_name}.tiles.zoom_level
    AND filtered_tiles.tile_column = {src_db_name}.tiles.tile_column
    AND filtered_tiles.tile_row = {src_db_name}.tiles.tile_row
LIMIT 1;
                "
                );
                debug!("Executing check_statement: {:?}", check_statement);
                c.query_row(check_statement, [], |row| {
                    let r: i64 = row.get(0)?;

                    Ok(r)
                })
            })
            .await
            .map_err(UtilesError::SqliteError)?;
        Ok(has_conflict > 0)
    }

    pub(super) async fn copy_tiles_stream(
        &self,
        src_db: &MbtilesClientAsync,
        dst_db: MbtilesClientAsync,
    ) -> UtilesResult<usize> {
        // detach src db if it is attached
        if let Err(e) = dst_db.detach_db("src").await {
            warn!("Error detaching src db: {:?}", e);
        } else {
            debug!("Detached src db");
        }

        // let where_clause = self.cfg.mbtiles_sql_where()?;
        let sql_query = self.cfg.tiles_stream_query()?;
        let dst_mbt_sync = Mbtiles::open_existing(dst_db.filepath()).map_err(|e| {
            error!("Error opening dst db: {:?}", e);
            e
        })?;
        dst_db.close().await.map_err(|e| {
            error!("Error closing dst db: {:?}", e);
            e
        })?;
        let stream_o_tiles = src_db.tiles_stream(Some(&*sql_query))?;

        let (tx2writer, rx) = tokio::sync::mpsc::channel(100);

        let process_tiles = tokio::spawn(async move {
            stream_o_tiles
                .for_each_concurrent(8, |(tile, tile_data)| {
                    let tx_writer = tx2writer.clone();
                    async move {
                        let hash_res = tokio::task::spawn_blocking(move || {
                            let hash = xxh64_be_hex_upper(&tile_data);
                            (tile_data, hash)
                        })
                        .await;
                        match hash_res {
                            Err(je) => {
                                warn!("hash-join-error: {:?}", je);
                            }
                            Ok((tile_data, hash_hex)) => {
                                if let Err(e) = tx_writer
                                    .send((tile, tile_data, Some(hash_hex)).into())
                                    .await
                                {
                                    warn!("send_res: {:?}", e);
                                }
                            }
                        }
                    }
                })
                .await;
        });
        let mut writer = MbtStreamWriterSync {
            mbt: dst_mbt_sync,
            stream: ReceiverStream::new(rx),
            on_conflict: self.cfg.istrat,
            stats: MbtWriterStats::default(),
        };
        let write_task = writer.write();
        let (process_tiles_res, write_task_res) =
            tokio::join!(process_tiles, write_task);
        if let Err(e) = process_tiles_res {
            error!("process_tiles_res: {:?}", e);
        }
        if let Err(e) = write_task_res {
            error!("write_task_res: {:?}", e);
        }
        Ok(0)
    }

    // pub(crate) async fn copy_tiles_attach(
    //     &self,
    //     dst_db: &MbtilesClientAsync,
    // ) -> UtilesResult<usize> {
    // }

    pub(super) async fn run(&self) -> UtilesResult<()> {
        warn!("mbtiles-2-mbtiles copy is a WIP");
        // doing preflight check
        debug!("Preflight check");
        let preflight = self.preflight_check().await?;
        info!(
            "Copying from {:?} ({}) -> {:?} {}",
            preflight.src_db.filepath(),
            preflight.src_db_type,
            preflight.dst_db.filepath(),
            preflight.dst_db_type
        );
        let dst_db = preflight.dst_db;
        let src_db_name = "src";
        let src_db_path = self.cfg.src_dbpath_str();
        debug!("Attaching src db ({:?}) to dst db...", src_db_path);
        dst_db.attach_db(&src_db_path, src_db_name).await?;
        debug!("OK: Attached src db");

        // ====================================================================
        // CHECK FOR CONFLICT (IF REQUIRED)
        // ====================================================================
        if preflight.check_conflict {
            info!("No conflict strategy provided; checking for conflict");
            let has_conflict = self.check_conflict(&dst_db).await?;
            if has_conflict {
                warn!("Conflict detected!");
                return Err(UtilesCopyError::Conflict(
                    "Conflict detected, no on-duplicate condition provided".to_string(),
                )
                .into());
            }
        } else if preflight.dst_is_new {
            debug!("dst db is new; not checking for conflict");
        } else {
            debug!(
                "No check required for conflict strategy: {}",
                self.cfg.istrat.to_string()
            );
        }

        // ====================================================================
        // COPY TILES (STREAM) EXPERIMENTAL
        // ====================================================================
        if self.cfg.stream {
            info!(
                "Copying tiles via stream: {:?} -> {:?}",
                self.cfg.src, self.cfg.dst
            );
            let start = std::time::Instant::now();
            let n_tiles_inserted =
                self.copy_tiles_stream(&preflight.src_db, dst_db).await?;
            let elapsed = start.elapsed();
            info!(
                "Copied {} tiles from {:?} -> {:?} in {:?}",
                n_tiles_inserted, self.cfg.src, self.cfg.dst, elapsed
            );
            return Ok(());
        }

        // ====================================================================
        // COPY TILES
        // ====================================================================
        info!("Copying tiles: {:?} -> {:?}", self.cfg.src, self.cfg.dst);
        let start = std::time::Instant::now();
        let n_tiles_inserted = self.copy_tiles_with_attach(&dst_db).await?;
        let elapsed = start.elapsed();
        info!(
            "Copied {} tiles from {:?} -> {:?} in {:?}",
            n_tiles_inserted, self.cfg.src, self.cfg.dst, elapsed
        );

        // ====================================================================
        // COPY METADATA
        // ====================================================================
        if let Some(src_db_metadata) = preflight.src_db_metadata {
            if preflight.dst_is_new {
                let n_metadata_inserted =
                    self.set_metadata(&dst_db, src_db_metadata).await?;
                debug!("n_metadata_inserted: {:?}", n_metadata_inserted);
            }
        }

        // update metadata minzoom and maxzoom
        dst_db.update_minzoom_maxzoom().await?;
        dst_db
            .metadata_set("dbtype", preflight.dst_db_type.as_str())
            .await?;
        if preflight.dst_db_type != MbtType::Flat {
            dst_db
                .metadata_set(
                    "tileid",
                    self.cfg.hash.unwrap_or_default().to_string().as_str(),
                )
                .await?;
        }
        debug!("Detaching src db...");
        dst_db.detach_db(src_db_name).await?;
        debug!("Detached src db!");
        Ok(())
    }
}
