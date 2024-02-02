use std::error::Error;
use std::path::PathBuf;

use async_sqlite::{JournalMode, Pool, PoolBuilder};
use async_trait::async_trait;
use tilejson::TileJSON;
use tracing::error;

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;

use crate::errors::UtilesResult;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::db_fspath::DbFspath;
use crate::utilesqlite::mbtiles::{mbtiles_metadata, query_zxy};
use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::UtilesError;

// #[derive(Debug)]
pub struct MbtilesAsyncSqlitePool {
    pub dbpath: DbFspath,
    pub pool: Pool,
}

#[async_trait]
impl MbtilesAsync for MbtilesAsyncSqlitePool {
    async fn open(path: &str) -> UtilesResult<Self> {
        let pool = PoolBuilder::new()
            .path(path)
            .journal_mode(JournalMode::Wal)
            .open()
            .await?;
        Ok(MbtilesAsyncSqlitePool {
            pool,
            dbpath: DbFspath::new(path),
        })
    }

    fn filepath(&self) -> &str {
        &self.dbpath.fspath
    }

    fn filename(&self) -> &str {
        &self.dbpath.filename
    }

    async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
        let metadata = self.metadata_rows().await?;
        let tj = metadata2tilejson(metadata);
        match tj {
            Ok(t) => Ok(t),
            Err(e) => {
                error!("Error parsing metadata to TileJSON: {}", e);
                Err(e)
            }
        }
    }

    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>> {
        let r = self
            .pool
            .conn(|conn| mbtiles_metadata(conn))
            .await
            .map_err(|e| UtilesError::AsyncSqliteError(e));
        match r {
            Ok(mdrows) => Ok(mdrows),
            Err(e) => Err(e.into()),
        }
    }
    //
    // async fn query_tile<T: TileLike>(&self, tile: T) -> UtilesResult<Option<Vec<u8>>> {
    //     let r = self.pool.conn(
    //         |conn| {
    //             Ok(query_tile(conn, tile))
    //         }
    //     ).await?;
    //     match r {
    //         Ok(tile) => Ok(tile),
    //         Err(e) => Err(e.into()),
    //     }
    // }

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        let r = self
            .pool
            .conn(move |conn| query_zxy(conn, z, x, y))
            .await
            .map_err(|e| UtilesError::AsyncSqliteError(e));
        match r {
            Ok(tile) => Ok(tile),
            Err(e) => Err(e.into()),
        }
    }
}
