use async_sqlite::{Client, ClientBuilder, Pool, PoolBuilder};
use async_trait::async_trait;
use futures::TryFutureExt;
use rusqlite::{Connection, OpenFlags};
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use std::str::FromStr;
use tilejson::TileJSON;
use tracing::{debug, error, info, warn};

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::mbt::mbtiles::{
    has_metadata_table_or_view, has_tiles_table_or_view, has_zoom_row_col_index,
    has_zxy, init_mbtiles, mbtiles_metadata, mbtiles_metadata_row,
    metadata_duplicate_key_values, metadata_json, minzoom_maxzoom, query_zxy,
    tiles_count, tiles_is_empty,
};
use crate::mbt::mbtiles_async::MbtilesAsync;
use crate::mbt::query::query_mbtiles_type;
use crate::mbt::zxyify::zxyify;
use crate::mbt::{
    MbtMetadataRow, MbtType, MbtilesMetadataJson, MbtilesStats, MetadataChangeFromTo,
    MinZoomMaxZoom, query_mbt_stats,
};
use crate::sqlite::{
    AsyncSqliteConn, AsyncSqliteConnMut, RowsAffected, SqliteError, journal_mode,
    magic_number, pragma_encoding,
};
use crate::sqlite::{DbPath, DbPathTrait, pathlike2dbpath};
use crate::sqlite_utiles::register_utiles_sqlite;
use crate::utilejson::metadata2tilejson;
use utiles_core::BBox;
use utiles_core::tile_type::{TileFormat, TileKind};

#[derive(Clone)]
pub struct MbtilesClientAsync {
    pub dbpath: DbPath,
    pub mbtype: MbtType,
    pub client: Client,
}

#[derive(Clone)]
pub struct MbtilesPoolAsync {
    pub dbpath: DbPath,
    pub mbtype: MbtType,
    pub pool: Pool,
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for MbtilesPoolAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //     use the dbpath to debug
        f.debug_struct("MbtilesAsyncSqlitePool")
            .field("fspath", &self.dbpath.fspath)
            .finish()
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for MbtilesClientAsync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MbtilesAsyncSqliteClient")
            .field("fspath", &self.dbpath.fspath)
            .finish()
    }
}

#[async_trait]
impl AsyncSqliteConn for MbtilesClientAsync {
    async fn conn<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn(func).await.map_err(SqliteError::from)
    }
}

#[async_trait]
impl AsyncSqliteConn for MbtilesPoolAsync {
    async fn conn<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.pool.conn(func).await.map_err(SqliteError::from)
    }
}

#[async_trait]
impl AsyncSqliteConnMut for MbtilesClientAsync {
    async fn conn_mut<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn_mut(func).await.map_err(SqliteError::from)
    }
}

#[async_trait]
impl AsyncSqliteConnMut for MbtilesPoolAsync {
    async fn conn_mut<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.pool.conn_mut(func).await.map_err(SqliteError::from)
    }
}

impl MbtilesClientAsync {
    pub async fn new(dbpath: DbPath, client: Client) -> UtilesResult<Self> {
        let mbtype = client.conn(query_mbtiles_type).await?;

        Ok(Self {
            dbpath,
            mbtype,
            client,
        })
    }
    pub async fn open_new<P: AsRef<Path>>(
        path: P,
        mbtype: Option<MbtType>,
    ) -> UtilesResult<Self> {
        let mbtype = mbtype.unwrap_or(MbtType::Flat);
        // make sure the path don't exist
        let dbpath = pathlike2dbpath(path)?;

        if dbpath.fspath_exists_async().await {
            Err(UtilesError::PathExistsError(dbpath.fspath))
        } else {
            debug!("Creating new mbtiles file with client: {}", dbpath);
            let client = ClientBuilder::new()
                .path(&dbpath.fspath)
                .flags(
                    OpenFlags::SQLITE_OPEN_READ_WRITE
                        | OpenFlags::SQLITE_OPEN_CREATE
                        | OpenFlags::SQLITE_OPEN_NO_MUTEX
                        | OpenFlags::SQLITE_OPEN_URI,
                )
                .open()
                .await?;
            debug!("db-type is: {:?}", mbtype);
            client
                .conn_mut(move |conn| {
                    init_mbtiles(conn, &mbtype)
                        // TODO: fix this and don't ignore the error...
                        .map_err(|_e| rusqlite::Error::InvalidQuery)?;
                    Ok(())
                })
                .await?;

            Self::new(dbpath, client).await
        }
    }
    pub async fn open<P: AsRef<Path>>(path: P) -> UtilesResult<Self> {
        let dbpath = pathlike2dbpath(path)?;
        debug!("Opening mbtiles file with client: {}", dbpath);
        let client = ClientBuilder::new()
            .path(&dbpath.fspath)
            .open()
            .await
            .map_err(|e| {
                debug!("Error opening mbtiles file: {}", e);
                e
            })?;
        Self::new(dbpath, client).await
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
            .await
            .map_err(|e| {
                debug!("Error opening existing mbtiles file: {}", e);
                e
            })?;
        Self::new(dbpath, client).await
    }

    pub async fn close(self) -> UtilesResult<()> {
        self.client.close().await?;
        Ok(())
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
            .map_err(|e| {
                debug!("Error opening readonly mbtiles file: {}", e);
                e
            })
            .await?;
        Self::new(dbpath, client).await
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

impl MbtilesPoolAsync {
    pub async fn new(dbpath: DbPath, pool: Pool) -> UtilesResult<Self> {
        let mbtype = pool.conn(query_mbtiles_type).await?;
        Ok(Self {
            dbpath,
            mbtype,
            pool,
        })
    }
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
        Self::new(dbpath, pool).await
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
        Self::new(dbpath, pool).await
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

impl DbPathTrait for MbtilesClientAsync {
    fn db_path(&self) -> &DbPath {
        &self.dbpath
    }
}

impl DbPathTrait for MbtilesPoolAsync {
    fn db_path(&self) -> &DbPath {
        &self.dbpath
    }
}

#[async_trait]
impl<T> MbtilesAsync for T
where
    T: AsyncSqliteConn + DbPathTrait + Debug,
{
    fn filepath(&self) -> &str {
        &self.db_path().fspath
    }

    fn filename(&self) -> &str {
        &self.db_path().filename
    }

    async fn register_utiles_sqlite_functions(&self) -> UtilesResult<()> {
        let r = self.conn(register_utiles_sqlite).await?;
        Ok(r)
    }

    async fn metadata_duplicate_key_values(
        &self,
    ) -> UtilesResult<Vec<(String, String, u32)>> {
        let r = self
            .conn(|conn| {
                let a = metadata_duplicate_key_values(conn)?;
                Ok(a)
            })
            .await?;
        Ok(r)
    }

    async fn metadata_update(
        &self,
        name: &str,
        value: &str,
    ) -> UtilesResult<Option<MetadataChangeFromTo>> {
        let current = self.metadata_row(name).await?;
        let change: Option<MetadataChangeFromTo> = {
            // there is a current value
            if let Some(c) = current {
                if c.value == value {
                    // if the value is the same
                    None
                } else {
                    // if the value is different
                    Some(MetadataChangeFromTo {
                        name: name.to_string(),
                        from: c.value.into(),
                        to: value.to_string().into(),
                    })
                }
            } else {
                Some(MetadataChangeFromTo {
                    name: name.to_string(),
                    from: None,
                    to: value.to_string().into(),
                })
            }
        };
        if let Some(c) = change {
            let _ = self.metadata_set(name, value).await?;
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    async fn update_minzoom_maxzoom(
        &self,
    ) -> UtilesResult<Option<Vec<MetadataChangeFromTo>>> {
        let query_minmax = self.query_minzoom_maxzoom().await?;
        if let Some(minmaxz) = query_minmax {
            let minzoom_change = self
                .metadata_update("minzoom", &minmaxz.minzoom.to_string())
                .await?;
            let maxzoom_change = self
                .metadata_update("maxzoom", &minmaxz.maxzoom.to_string())
                .await?;
            let changes = vec![minzoom_change, maxzoom_change];
            Ok(Some(changes.into_iter().flatten().collect()))
        } else {
            Ok(None)
        }
    }

    async fn is_mbtiles_like(&self) -> UtilesResult<bool> {
        let has_metadata_table_or_view = self.conn(has_metadata_table_or_view).await?;
        debug!("has-metadata-table-or-view: {}", has_metadata_table_or_view);
        let has_tiles_table_or_view = self.conn(has_tiles_table_or_view).await?;
        debug!("has-tiles-table-or-view: {}", has_tiles_table_or_view);
        if !has_metadata_table_or_view || !has_tiles_table_or_view {
            debug!("Not a mbtiles file: {}", self.filepath());
            return Ok(false);
        }
        Ok(true)
    }

    async fn is_mbtiles(&self) -> UtilesResult<bool> {
        let is_mbtiles_like = self.is_mbtiles_like().await?;
        if !is_mbtiles_like {
            return Ok(false);
        }
        let has_zoom_row_col_index = self.conn(has_zoom_row_col_index).await?;
        debug!(
            target: "is-mbtiles",
            "has_zoom_row_col_index: {}",
            has_zoom_row_col_index,
        );
        Ok(has_zoom_row_col_index)
    }

    async fn assert_mbtiles(&self) -> UtilesResult<()> {
        let is_mbtiles = self.is_mbtiles().await?;

        if is_mbtiles {
            Ok(())
        } else {
            Err(UtilesError::NotMbtilesLike(self.filepath().to_string()))
        }
    }

    async fn magic_number(&self) -> UtilesResult<u32> {
        let magic_number = self.conn(magic_number).await?;
        Ok(magic_number)
    }

    async fn pragma_encoding(&self) -> UtilesResult<String> {
        self.conn(pragma_encoding).await.map_err(UtilesError::from)
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

    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtMetadataRow>> {
        let metadata = self
            .conn(mbtiles_metadata)
            .await
            .map_err(UtilesError::SqliteError)?;
        Ok(metadata)
    }

    async fn metadata_json(&self) -> UtilesResult<MbtilesMetadataJson> {
        let data = self.conn(metadata_json).await?;
        Ok(data)
    }

    // async fn attach(&self, path: &str, dbname: &str) -> UtilesResult<usize> {
    //     let path_string = path.to_string();
    //     let as_string = dbname.to_string();
    //     let rows = self
    //         .conn(move |conn| {
    //             conn.execute("ATTACH DATABASE ?1 AS ?2", [&path_string, &as_string])
    //         })
    //         .await
    //         .map_err(UtilesError::AsyncSqliteError)?;
    //     Ok(rows)
    // }
    //
    // async fn detach(&self, dbname: &str) -> UtilesResult<usize> {
    //     let as_string = dbname.to_string();
    //     let rows = self
    //         .conn(move |conn| conn.execute("DETACH DATABASE ?1", [&as_string]))
    //         .await
    //         .map_err(UtilesError::AsyncSqliteError)?;
    //     Ok(rows)
    // }

    async fn metadata_row(&self, name: &str) -> UtilesResult<Option<MbtMetadataRow>> {
        let name_str = name.to_string();
        let row = self
            .conn(move |conn| mbtiles_metadata_row(conn, &name_str))
            .await
            .map_err(UtilesError::SqliteError)?;
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
            .map_err(UtilesError::SqliteError)?;
        Ok(rows)
    }

    async fn tiles_is_empty(&self) -> UtilesResult<bool> {
        let tiles_is_empty = self
            .conn(tiles_is_empty)
            .await
            .map_err(UtilesError::SqliteError)?;
        Ok(tiles_is_empty)
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
    async fn has_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<bool> {
        let res = self
            .conn(move |conn| has_zxy(conn, z, x, y))
            .await
            .map_err(UtilesError::SqliteError)?;
        Ok(res)
    }

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        let tile = self
            .conn(move |conn| query_zxy(conn, z, x, y))
            .await
            .map_err(UtilesError::SqliteError)?;
        Ok(tile)
    }

    async fn query_minzoom_maxzoom(&self) -> UtilesResult<Option<MinZoomMaxZoom>> {
        let t = self
            .conn(minzoom_maxzoom)
            .await
            .map_err(UtilesError::SqliteError)?;
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

    async fn query_mbt_type(&self) -> UtilesResult<MbtType> {
        let mbt = self.conn(query_mbtiles_type).await?;
        Ok(mbt)
    }

    async fn query_metadata_value(&self, name: &str) -> UtilesResult<Option<String>> {
        let row = self.metadata_row(name).await?;
        match row {
            Some(r) => Ok(Some(r.value)),
            None => Ok(None),
        }
    }

    async fn query_metadata_format(&self) -> UtilesResult<Option<String>> {
        self.query_metadata_value("format").await
    }

    async fn query_tilekind(&self) -> UtilesResult<TileKind> {
        let format_str = self.query_metadata_format().await?;
        if let Some(format_str) = format_str {
            let kind = TileFormat::from_str(&format_str)
                .map(|f| f.kind())
                .unwrap_or(TileKind::Unknown);
            Ok(kind)
        } else {
            Ok(TileKind::Unknown)
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

    async fn zxyify(&self) -> UtilesResult<Vec<RowsAffected>> {
        let r = self.conn(zxyify).await?;
        Ok(r)
    }

    async fn mbt_stats(&self, full: Option<bool>) -> UtilesResult<MbtilesStats> {
        self.conn(move |conn| {
            let r = query_mbt_stats(conn, full);
            Ok(r)
        })
        .await?
    }

    async fn tiles_count(&self) -> UtilesResult<usize> {
        self.conn(tiles_count).await.map_err(UtilesError::from)
    }
}
