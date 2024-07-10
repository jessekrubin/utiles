#![allow(dead_code)]

use crate::copy::CopyConfig;
use crate::errors::{UtilesCopyError, UtilesResult};
use crate::mbt::MbtType;
use crate::sqlite::{AsyncSqliteConn, Sqlike3Async};
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesError;
use tracing::field::debug;
use tracing::{debug, info, warn};
use utiles_core::UtilesCoreError;

#[derive(Debug)]
pub struct CopyPasta {
    pub cfg: CopyConfig,
}

impl CopyPasta {
    pub fn new(cfg: CopyConfig) -> UtilesResult<CopyPasta> {
        cfg.check()?;
        // sanity check stuff here...
        Ok(Self { cfg })
    }

    pub async fn get_src_db(&self) -> UtilesResult<MbtilesAsyncSqliteClient> {
        // do the thing
        let src_db = MbtilesAsyncSqliteClient::open_existing(&self.cfg.src).await?;

        debug!("src_db: {:?}", src_db);
        Ok(src_db)
    }

    pub async fn get_src_dbtype(&self) -> UtilesResult<MbtType> {
        let src_db = self.get_src_db().await?;
        Ok(src_db.mbtype)
    }

    /// Returns the destination db and a bool indicating if it was created
    pub async fn get_dst_db(
        &self,
    ) -> UtilesResult<(MbtilesAsyncSqliteClient, bool, MbtType)> {
        // if the dst is a file... we gotta get it...
        let dst_db_res = MbtilesAsyncSqliteClient::open_existing(&self.cfg.dst).await;
        let (dst_db, is_new) = match dst_db_res {
            Ok(db) => (db, false),
            Err(e) => {
                debug!("dst_db_res: {:?}", e);
                debug!("Creating new db... {:?}", self.cfg.dst);
                // type is
                debug!("dbtype: {:?}", self.cfg.dbtype);
                let db = MbtilesAsyncSqliteClient::open_new(
                    &self.cfg.dst,
                    // todo!
                    self.cfg.dbtype,
                )
                .await?;
                (db, true)
            }
        };
        dst_db.register_utiles_sqlite_functions().await?;
        let db_type_queried = dst_db.query_mbt_type().await?;
        Ok((dst_db, is_new, db_type_queried))
    }

    pub async fn copy_metadata(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<usize> {
        let src_db = self.get_src_db().await?;
        let metadata_rows = src_db.metadata_json().await?.as_obj();
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

    pub async fn copy_tiles_zbox_flat(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
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

    pub async fn copy_tiles_zbox_hash(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
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

    pub async fn copy_tiles_zbox_norm(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
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

    pub async fn copy_tiles_zbox(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
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
        Ok(res)
    }

    pub fn preflight_check(&self) -> UtilesResult<()> {
        // do the thing
        debug!("Preflight check: {:?}", self.cfg);

        Ok(())
    }

    pub async fn check_conflict(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
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
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(has_conflict > 0)
    }

    pub async fn run(&self) -> UtilesResult<()> {
        warn!("mbtiles-2-mbtiles copy is a WIP");
        // doing preflight check
        debug!("Preflight check");
        self.preflight_check()?;
        let (dst_db, is_new, db_type) = self.get_dst_db().await?;

        let src_db_name = "src";

        let src_db_path = self.cfg.src_dbpath_str();

        debug!("Attaching src db ({:?}) to dst db...", src_db_path);
        dst_db.attach_db(&src_db_path, src_db_name).await?;
        debug!("OK: Attached src db");
        if !is_new && self.cfg.istrat.requires_check() {
            info!("No conflict strategy provided; checking for conflict");
            let has_conflict = self.check_conflict(&dst_db).await?;
            if has_conflict {
                warn!("Conflict detected!");
                return Err(UtilesCopyError::Conflict(
                    "Conflict detected, no on-duplicate condition provided".to_string(),
                )
                .into());
            }
        } else {
            if is_new {
                debug!("dst db is new; not checking for conflict");
            } else {
                debug!(
                    "No check required for conflict strategy: {}",
                    self.cfg.istrat.to_string()
                );
            }
        }

        let n_tiles_inserted = self.copy_tiles_zbox(&dst_db).await?;
        debug!("n_tiles_inserted: {:?}", n_tiles_inserted);

        if is_new {
            let n_metadata_inserted = self.copy_metadata(&dst_db).await?;
            debug!("n_metadata_inserted: {:?}", n_metadata_inserted);
        }

        dst_db.metadata_set("dbtype", db_type.as_str()).await?;
        if db_type == MbtType::Hash || db_type == MbtType::Norm {
            dst_db
                .metadata_set(
                    "tileid",
                    self.cfg
                        .hash
                        .unwrap_or_default()
                        .to_string()
                        .to_string()
                        .as_str(),
                )
                .await?;
        }

        debug!("Detaching src db...");
        dst_db.detach_db(src_db_name).await?;
        debug!("Detached src db!");
        Ok(())
    }
}
