use std::fmt;
use std::fmt::Debug;
use std::path::Path;

use crate::errors::UtilesResult;
use crate::mbt::query::query_mbtiles_type;
use crate::mbt::{MbtMetadataRow, MbtType, MinZoomMaxZoom};
use crate::sqlite::{journal_mode, magic_number};
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::dbpath::{pathlike2dbpath, DbPath, DbPathTrait};
use crate::utilesqlite::mbtiles::{
    has_metadata_table_or_view, has_tiles_table_or_view, has_zoom_row_col_index,
    mbtiles_metadata, mbtiles_metadata_row, minzoom_maxzoom, query_zxy,
    register_utiles_sqlite_functions, tiles_is_empty,
};
use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::UtilesError;
use async_sqlite::{
    Client, ClientBuilder, Error as AsyncSqliteError, Pool, PoolBuilder,
};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use tilejson::TileJSON;
use tracing::{debug, error, info, warn};
use utiles_core::BBox;

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

#[allow(clippy::missing_fields_in_debug)]
impl Debug for MbtilesAsyncSqlitePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //     use the dbpath to debug
        f.debug_struct("MbtilesAsyncSqlitePool")
            .field("fspath", &self.dbpath.fspath)
            .finish()
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for MbtilesAsyncSqliteClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MbtilesAsyncSqliteClient")
            .field("fspath", &self.dbpath.fspath)
            .finish()
    }
}

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
    pub async fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(path)?;
        debug!("Opening mbtiles file with client: {}", dbpath);
        let client = ClientBuilder::new().path(&dbpath.fspath).open().await?;
        Ok(MbtilesAsyncSqliteClient { dbpath, client })
    }

    pub async fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(path)?;
        debug!("Opening existing mbtiles file with client: {}", dbpath);
        let client = ClientBuilder::new()
            .path(&dbpath.fspath)
            .flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX
                    | OpenFlags::SQLITE_OPEN_URI,
            )
            .open()
            .await?;
        Ok(MbtilesAsyncSqliteClient { dbpath, client })
    }
    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        let dbpath = pathlike2dbpath(path)?;
        debug!("Opening readonly mbtiles file with client: {}", dbpath);
        let client = ClientBuilder::new()
            .path(&dbpath.fspath)
            .flags(flags)
            .open()
            .await?;
        Ok(MbtilesAsyncSqliteClient { dbpath, client })
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
        let dbpath = pathlike2dbpath(path)?;
        info!("Opening mbtiles file with pool: {}", dbpath);
        let pool = PoolBuilder::new()
            .path(&dbpath.fspath)
            .flags(flags)
            .num_conns(2)
            .open()
            .await?;
        Ok(MbtilesAsyncSqlitePool { dbpath, pool })
    }

    pub async fn open_existing<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(path)?;
        info!("Opening existing mbtiles file with pool: {}", dbpath);
        let pool = PoolBuilder::new()
            .path(&dbpath.fspath)
            .flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX
                    | OpenFlags::SQLITE_OPEN_URI,
            )
            .num_conns(2)
            .open()
            .await?;
        Ok(MbtilesAsyncSqlitePool { dbpath, pool })
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
    T: AsyncSqlite + DbPathTrait + Debug,
{
    fn filepath(&self) -> &str {
        &self.db_path().fspath
    }

    fn filename(&self) -> &str {
        &self.db_path().filename
    }

    async fn register_utiles_sqlite_functions(&self) -> UtilesResult<()> {
        let r = self.conn(register_utiles_sqlite_functions).await?;
        Ok(r)
    }

    #[tracing::instrument]
    async fn is_mbtiles(&self) -> UtilesResult<bool> {
        let has_metadata_table_or_view = self.conn(has_metadata_table_or_view).await?;
        debug!("has-metadata-table-or-view: {}", has_metadata_table_or_view);
        let has_tiles_table_or_view = self.conn(has_tiles_table_or_view).await?;
        debug!("has-tiles-table-or-view: {}", has_tiles_table_or_view);
        if !has_metadata_table_or_view || !has_tiles_table_or_view {
            debug!("Not a mbtiles file: {}", self.filepath());
            return Ok(false);
        }
        // assert tiles is not empty
        let tiles_is_empty = self
            .conn(tiles_is_empty)
            .await
            .map_err(UtilesError::AsyncSqliteError);
        if let Ok(true) = tiles_is_empty {
            debug!("Empty tiles table: {}", self.filepath());
            return Ok(false);
        }
        if let Err(e) = tiles_is_empty {
            error!("Error checking if tiles table is empty: {}", e);
            return Err(e);
        }

        let has_zoom_row_col_index = self.conn(has_zoom_row_col_index).await?;

        debug!(
            target: "is-mbtiles",
            "has_zoom_row_col_index: {}",
            has_zoom_row_col_index,
        );
        Ok(has_zoom_row_col_index)
    }

    async fn tiles_is_empty(&self) -> UtilesResult<bool> {
        let tiles_is_empty = self
            .conn(tiles_is_empty)
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(tiles_is_empty)
    }

    async fn magic_number(&self) -> UtilesResult<u32> {
        let magic_number = self.conn(magic_number).await?;
        Ok(magic_number)
    }

    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtMetadataRow>> {
        let metadata = self
            .conn(mbtiles_metadata)
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(metadata)
    }

    async fn metadata_row(&self, name: &str) -> UtilesResult<Option<MbtMetadataRow>> {
        let name_str = name.to_string();
        let row = self
            .conn(move |conn| mbtiles_metadata_row(conn, &name_str))
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(row)
    }

    async fn metadata_set(&self, name: &str, value: &str) -> UtilesResult<usize> {
        let name_str = name.to_string();
        let value_str = value.to_string();
        let rows = self
            .conn(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)",
                    [&name_str, &value_str],
                )
            })
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(rows)
    }

    async fn metadata_minzoom(&self) -> UtilesResult<Option<u8>> {
        let minzoom = self.metadata_row("minzoom").await?;
        match minzoom {
            Some(m) => {
                let mz = m.value.parse::<u8>().map_err(UtilesError::from);
                match mz {
                    Ok(z) => Ok(Some(z)),
                    Err(e) => {
                        error!("Error parsing minzoom: {}", e);
                        Err(e)
                    }
                }
            }
            None => Ok(None),
        }
    }

    async fn metadata_maxzoom(&self) -> UtilesResult<Option<u8>> {
        let maxzoom = self.metadata_row("maxzoom").await?;
        match maxzoom {
            Some(m) => {
                let mz = m.value.parse::<u8>().map_err(UtilesError::from);
                match mz {
                    Ok(z) => Ok(Some(z)),
                    Err(e) => {
                        error!("Error parsing maxzoom: {}", e);
                        Err(e)
                    }
                }
            }
            None => Ok(None),
        }
    }

    async fn tilejson(&self) -> UtilesResult<TileJSON> {
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

    async fn bbox(&self) -> UtilesResult<BBox> {
        let tilejson = self.tilejson().await?;
        let bounding = tilejson.bounds;
        match bounding {
            Some(bounds) => Ok(BBox::new(
                bounds.left,
                bounds.bottom,
                bounds.right,
                bounds.top,
            )),
            None => Err(UtilesError::ParsingError(
                "Error parsing metadata to BBox: no bounds".into(),
            )),
        }
    }
    async fn query_minzoom_maxzoom(&self) -> UtilesResult<Option<MinZoomMaxZoom>> {
        let t = self
            .conn(minzoom_maxzoom)
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        Ok(t)
    }

    async fn tilejson_ext(&self) -> UtilesResult<TileJSON> {
        let mut metadata = self.metadata_rows().await?;
        // if no 'minzoom' or 'maxzoom' are found we gotta query them...

        // check if minzoom or maxzoom are missing
        let minzoom_value = metadata.iter().find(|m| m.name == "minzoom");
        let maxzoom_value = metadata.iter().find(|m| m.name == "maxzoom");

        // check if minzoom and maxzoom are present and valid
        let minzoom_maxzoom_status = match (minzoom_value, maxzoom_value) {
            (Some(minzoom), Some(maxzoom)) => {
                let minzoom = minzoom.value.parse::<u8>();
                let maxzoom = maxzoom.value.parse::<u8>();
                matches!((minzoom, maxzoom), (Ok(_), Ok(_)))
            }
            _ => false,
        };

        if !minzoom_maxzoom_status {
            warn!("minzoom/maxzoom missing from metadata: {}", self.filepath());
            let minmax = self.query_minzoom_maxzoom().await?;
            match minmax {
                Some(mm) => {
                    let minzoom = MbtMetadataRow {
                        name: "minzoom".to_string(),
                        value: mm.minzoom.to_string(),
                    };
                    let maxzoom = MbtMetadataRow {
                        name: "maxzoom".to_string(),
                        value: mm.maxzoom.to_string(),
                    };
                    metadata.push(minzoom);
                    metadata.push(maxzoom);
                }
                None => {
                    error!("Unable to query minzoom maxzoom");
                }
            }
        }
        let tj = metadata2tilejson(metadata);
        match tj {
            Ok(t) => Ok(t),
            Err(e) => {
                error!("Error parsing metadata to TileJSON: {}", e);
                Err(e)
            }
        }
    }

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        let tile = self
            .conn(move |conn| query_zxy(conn, z, x, y))
            .await
            .map_err(UtilesError::AsyncSqliteError)?;

        Ok(tile)
    }

    async fn query_mbt_type(&self) -> UtilesResult<MbtType> {
        self.conn(|conn| Ok(query_mbtiles_type(conn))).await?
    }
}

// =============================================================
// =============================================================
// NON GENERIC IMPLEMENTATION
// =============================================================
// =============================================================

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
