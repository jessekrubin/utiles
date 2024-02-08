use std::error::Error;
use std::path::Path;

use async_sqlite::{Client, ClientBuilder, JournalMode, Pool, PoolBuilder};
use async_trait::async_trait;
use rusqlite::{OpenFlags, Result};
use tilejson::TileJSON;
use tracing::{error, info};

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;

use crate::errors::UtilesResult;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::dbpath::DbPath;
use crate::utilesqlite::mbtiles::{mbtiles_metadata, query_zxy};
use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::utilesqlite::squealite::{journal_mode, magic_number};
use crate::UtilesError;

pub struct MbtilesAsyncSqliteClient {
    pub dbpath: DbPath,
    pub client: Client,
}

pub struct MbtilesAsyncSqlitePool {
    pub dbpath: DbPath,
    pub pool: Pool,
}

impl MbtilesAsyncSqliteClient {
    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        let dbpath = DbPath::new(path.as_ref().to_str().unwrap());
        info!("Opening mbtiles file: {}", dbpath);
        let client = ClientBuilder::new().path(path).flags(flags).open().await?;
        Ok(MbtilesAsyncSqliteClient { client, dbpath })
    }

    pub async fn journal_mode_wal(self) -> UtilesResult<Self> {
        self.client
            .conn(|conn| conn.pragma_update(None, "journal_mode", "WAL"))
            .await?;
        Ok(self)
    }

    /// Return the current journal mode
    pub async fn journal_mode(self) -> UtilesResult<String> {
        let jm = self.client.conn(journal_mode).await?;
        Ok(jm)
    }
}

impl MbtilesAsyncSqlitePool {
    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        let dbpath = DbPath::new(path.as_ref().to_str().unwrap());
        info!("Opening mbtiles file: {}", dbpath);
        let pool = PoolBuilder::new().path(path).flags(flags).open().await?;
        Ok(MbtilesAsyncSqlitePool { pool, dbpath })
    }

    pub async fn journal_mode_wal(self) -> UtilesResult<Self> {
        self.pool
            .conn(move |conn| conn.pragma_update(None, "journal_mode", "WAL"))
            .await?;
        Ok(self)
    }

    /// Return the current journal mode
    pub async fn journal_mode(self) -> UtilesResult<String> {
        let journal_mode = self.pool.conn(journal_mode).await?;
        Ok(journal_mode)
    }
}

// #[async_trait]
// impl Sqlike3Async for MbtilesAsyncSqlitePool {
//     async fn magic_number(&self) -> RusqliteResult<u32> {
//         let r = self.pool.conn(magic_number).await;
//         r
//     }
//     async fn is_empty_db(&self) -> RusqliteResult<bool> {
//         self.pool.conn(is_empty_db)
//     }
//     async fn vacuum(&self) -> RusqliteResult<usize> {
//         self.pool.conn(vacuum)
//     }
//     async fn analyze(&self) -> RusqliteResult<usize> {
//         self.pool.conn(analyze)
//     }
// }

#[async_trait]
impl MbtilesAsync for MbtilesAsyncSqlitePool {
    fn filepath(&self) -> &str {
        &self.dbpath.fspath
    }

    fn filename(&self) -> &str {
        &self.dbpath.filename
    }

    async fn open(path: &str) -> UtilesResult<Self> {
        let pool = PoolBuilder::new()
            .path(path)
            .journal_mode(JournalMode::Wal)
            .open()
            .await?;
        Ok(MbtilesAsyncSqlitePool {
            pool,
            dbpath: DbPath::new(path),
        })
    }

    async fn magic_number(&self) -> UtilesResult<u32> {
        self.pool
            .conn(magic_number)
            .await
            .map_err(UtilesError::AsyncSqliteError)
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
        self.pool
            .conn(mbtiles_metadata)
            .await
            .map_err(UtilesError::AsyncSqliteError)
    }

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        self.pool
            .conn(move |conn| query_zxy(conn, z, x, y))
            .await
            .map_err(UtilesError::AsyncSqliteError)
    }
}
