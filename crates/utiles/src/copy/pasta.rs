#![allow(dead_code)]

use tracing::{debug, warn};

use crate::copy::CopyConfig;
use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::utilesqlite::mbtiles_async_sqlite::AsyncSqlite;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesError;

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

    pub fn preflight_check(&self) -> UtilesResult<()> {
        // do the thing
        debug!("Preflight check: {:?}", self.cfg);

        Ok(())
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

    pub async fn get_dst_db(&self) -> UtilesResult<MbtilesAsyncSqliteClient> {
        // if the dst is a file... we gotta get it...
        let dst_db_res = MbtilesAsyncSqliteClient::open_existing(&self.cfg.dst).await;

        let dst_db = match dst_db_res {
            Ok(db) => db,
            Err(e) => {
                debug!("dst_db_res: {:?}", e);
                MbtilesAsyncSqliteClient::open_new(
                    &self.cfg.dst,
                    // todo!
                    None,
                )
                .await?
            }
        };
        Ok(dst_db)
    }

    pub async fn copy_metadata(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<usize> {
        let src_db = self.get_src_db().await?;
        let metadata_rows = src_db.metadata_rows().await?;
        let mut n_metadata_inserted = 0;
        for row in metadata_rows {
            let res = dst_db.metadata_set(&row.name, &row.value).await?;
            n_metadata_inserted += res;
        }
        Ok(n_metadata_inserted)
    }

    pub async fn copy_tiles_zbox(
        &self,
        dst_db: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<usize> {
        let src_db_name = "src";
        let where_clause = self.cfg.mbtiles_sql_where()?;
        let n_tiles_inserted = dst_db.conn(
            move |x| {
                let insert_statement = &format!(
                        "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) SELECT zoom_level, tile_column, tile_row, tile_data FROM {src_db_name}.tiles {where_clause}");

                debug!("Executing tiles insert: {:?}", insert_statement);
                
                x.execute(
                    insert_statement,
                    [],
                )
            }
        ).await.map_err(
            UtilesError::AsyncSqliteError
        )?;
        if n_tiles_inserted == 0 {
            warn!("No tiles inserted!");
        } else {
            debug!("n_tiles_inserted: {:?}", n_tiles_inserted);
        }
        Ok(n_tiles_inserted)
    }

    pub async fn run(&self) -> UtilesResult<()> {
        warn!("mbtiles-2-mbtiles copy is a WIP");
        // doing preflight check
        debug!("Preflight check");
        self.preflight_check()?;

        let dst_db = self.get_dst_db().await?;

        let src_db_name = "src";

        // let src_db_path = self.cfg.src.to_str();
        // if src_db_path.is_none() {
        //     return Err(UtilesError::PathConversionError(
        //         "src db path conversion error".to_string(),
        //     ));
        // }
        let src_db_path = self.cfg.src_dbpath_str();

        dst_db.attach(&src_db_path, src_db_name).await?;
        let n_tiles_inserted = self.copy_tiles_zbox(&dst_db).await?;

        debug!("n_tiles_inserted: {:?}", n_tiles_inserted);

        debug!("Detaching src db...");
        dst_db.detach(src_db_name).await?;
        debug!("Detached src db!");
        Ok(())
    }
}
