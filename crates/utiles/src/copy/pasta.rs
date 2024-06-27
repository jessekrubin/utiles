#![allow(dead_code)]

use log::warn;
use tracing::debug;

use crate::copy::CopyConfig;
use crate::errors::UtilesResult;
use crate::mbt::MbtType;
use crate::utilesqlite::MbtilesAsyncSqliteClient;

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

    pub async fn run(&self) -> UtilesResult<()> {
        warn!("mbtiles-2-mbtiles copy is a WIP");
        // doing preflight check
        debug!("Preflight check");
        self.preflight_check()?;

        let dst_db = self.get_dst_db().await?;

        Ok(())
    }
}
