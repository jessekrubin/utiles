use std::error::Error;
use std::path::Path;

use async_sqlite::{
    Client, ClientBuilder, Error as AsyncSqliteError, Pool, PoolBuilder,
};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use tilejson::TileJSON;
use tracing::{debug, error, info};

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;

use crate::errors::UtilesResult;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::dbpath::{DbPath, DbPathTrait};
use crate::utilesqlite::mbtiles::{mbtiles_metadata, query_zxy};
use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::utilesqlite::squealite::{journal_mode, magic_number};

#[derive(Clone)]
pub struct MbtilesAsyncSqliteClient {
    pub dbpath: DbPath,
    pub client: Client,
}

#[derive(Clone)]
pub struct MbtilesAsyncSqlitePool {
    pub dbpath: DbPath,
    pub pool: Pool,
}

// #[async_trait]
// pub trait Sqlike3Async {
//     async fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self>
//         where
//             Self: Sized + Send; // Ensure Self is Send
//     // async fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self> where Self: Sized + Send;
// }
// impl<T> MbtilesAsync for T
//     where
//         T: AsyncSqlite,

#[async_trait]
pub trait AsyncSqlite: Send + Sync {
    async fn conn<F, T>(&self, func: F) -> Result<T, AsyncSqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static;
}

#[async_trait]
impl AsyncSqlite for MbtilesAsyncSqliteClient {
    async fn conn<F, T>(&self, func: F) -> Result<T, AsyncSqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn(func).await
    }
}

#[async_trait]
impl AsyncSqlite for MbtilesAsyncSqlitePool {
    async fn conn<F, T>(&self, func: F) -> Result<T, AsyncSqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.pool.conn(func).await
    }
}

impl MbtilesAsyncSqliteClient {
    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        let dbpath = DbPath::new(path.as_ref().to_str().unwrap());
        debug!("Opening mbtiles file with client: {}", dbpath);
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

// impl Client
// pub async fn conn<F, T>(&self, func: F) -> Result<T, Error> where     F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,     T: Send + 'static,
impl MbtilesAsyncSqlitePool {
    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        let dbpath = DbPath::new(path.as_ref().to_str().unwrap());
        info!("Opening mbtiles file with pool: {}", dbpath);
        let pool = PoolBuilder::new()
            .path(path)
            .flags(flags)
            .num_conns(2)
            .open()
            .await?;
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
// #[async_trait]
// impl MbtilesAsync for MbtilesAsyncSqliteClient {
//     // Implementation of other methods...
//
//     fn dbpath(&self) -> &DbPath {
//         &self.dbpath
//     }
//
//     // Correct the filepath method if necessary
//     fn filepath(&self) -> &str {
//         self.db_path().as_str() // Assuming DbPath has an as_str() method to get the filesystem path
//     }
//
//     // Ensure the filename method is correctly implemented
//     fn filename(&self) -> &str {
//         // You need to implement logic to extract the filename from the dbpath
//     }
// }

impl DbPathTrait for MbtilesAsyncSqliteClient {
    fn db_path(&self) -> &DbPath {
        &self.dbpath
    }
}

impl DbPathTrait for MbtilesAsyncSqlitePool {
    fn db_path(&self) -> &DbPath {
        &self.dbpath
    }
}

#[async_trait]
impl<T> MbtilesAsync for T
where
    T: AsyncSqlite + DbPathTrait,
{
    fn filepath(&self) -> &str {
        &self.db_path().fspath
    }

    fn filename(&self) -> &str {
        &self.db_path().filename
    }

    async fn magic_number(&self) -> UtilesResult<u32> {
        let magic_number = self.conn(magic_number).await?;
        Ok(magic_number)
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
        let metadata = self.conn(mbtiles_metadata).await?;
        Ok(metadata)
    }

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        let tile = self.conn(move |conn| query_zxy(conn, z, x, y)).await?;
        Ok(tile)
    }
}
//
// #[async_trait]
// impl MbtilesAsync for MbtilesAsyncSqliteClient {
//     fn filepath(&self) -> &str {
//         &self.dbpath.fspath
//     }
//
//     fn filename(&self) -> &str {
//         &self.dbpath.filename
//     }
//
//
//     async fn magic_number(&self) -> UtilesResult<u32> {
//         self.client
//             .conn(magic_number)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
//
//     async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
//         let metadata = self.metadata_rows().await?;
//         let tj = metadata2tilejson(metadata);
//         match tj {
//             Ok(t) => Ok(t),
//             Err(e) => {
//                 error!("Error parsing metadata to TileJSON: {}", e);
//                 Err(e)
//             }
//         }
//     }
//
//     async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>> {
//         self.client
//             .conn(mbtiles_metadata)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
//
//     async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
//         self.client
//             .conn(move |conn| query_zxy(conn, z, x, y))
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
// }
//
// #[async_trait]
// impl MbtilesAsync for MbtilesAsyncSqlitePool {
//     fn filepath(&self) -> &str {
//         &self.dbpath.fspath
//     }
//
//     fn filename(&self) -> &str {
//         &self.dbpath.filename
//     }
//
//     // async fn open(path: &str) -> UtilesResult<Self> {
//     //     let pool = PoolBuilder::new()
//     //         .path(path)
//     //         .journal_mode(JournalMode::Wal)
//     //         .open()
//     //         .await?;
//     //     Ok(MbtilesAsyncSqlitePool {
//     //         pool,
//     //         dbpath: DbPath::new(path),
//     //     })
//     // }
//
//     async fn magic_number(&self) -> UtilesResult<u32> {
//         self.pool
//             .conn(magic_number)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
//
//     async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
//         let metadata = self.metadata_rows().await?;
//         let tj = metadata2tilejson(metadata);
//         match tj {
//             Ok(t) => Ok(t),
//             Err(e) => {
//                 error!("Error parsing metadata to TileJSON: {}", e);
//                 Err(e)
//             }
//         }
//     }
//
//     async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>> {
//         self.pool
//             .conn(mbtiles_metadata)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
//
//     async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
//         self.pool
//             .conn(move |conn| query_zxy(conn, z, x, y))
//             .await
//             .map_err(UtilesError::AsyncSqliteError)
//     }
// }
