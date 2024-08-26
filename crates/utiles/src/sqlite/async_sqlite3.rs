use crate::fs_async::file_exists;
use crate::sqlite::sqlike3::Sqlike3Async;
use crate::sqlite::{
    analysis_limit, analyze, attach_db, detach_db, is_empty_db, pragma_freelist_count,
    pragma_index_list, pragma_page_count, pragma_page_size, pragma_page_size_set,
    pragma_table_list, vacuum, vacuum_into, DbPath, PragmaIndexListRow,
    PragmaTableListRow, SqliteError, SqliteResult,
};
use async_sqlite::{Client, ClientBuilder};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use std::fmt;
use std::fmt::Debug;
use std::path::Path;
use tracing::debug;

pub struct SqliteDbAsyncClient {
    pub dbpath: DbPath,
    pub client: Client,
}
#[allow(clippy::missing_fields_in_debug)]
impl Debug for SqliteDbAsyncClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //     use the dbpath to debug
        f.debug_struct("SqliteDbAsyncClient")
            .field("fspath", &self.dbpath.fspath)
            .finish()
    }
}

impl SqliteDbAsyncClient {
    pub async fn new(client: Client, dbpath: Option<DbPath>) -> SqliteResult<Self> {
        if let Some(dbpath) = dbpath {
            Ok(Self { dbpath, client })
        } else {
            let path = client
                .conn(|conn| {
                    let maybe_path = conn.path();

                    let path = maybe_path.unwrap_or("unknown").to_string();
                    Ok(path)
                })
                .await?;
            Ok(Self {
                client,
                dbpath: DbPath::new(&path),
            })
        }
    }

    pub async fn open<P: AsRef<Path>>(
        path: P,
        open_flags: Option<OpenFlags>,
    ) -> SqliteResult<Self> {
        debug!("Opening sqlite db with client: {}", path.as_ref().display());
        let client = ClientBuilder::new()
            .path(&path)
            .flags(open_flags.unwrap_or_default())
            .open()
            .await?;
        Ok({
            Self {
                dbpath: DbPath::new(path.as_ref().to_str().unwrap_or_default()),
                client,
            }
        })
    }

    pub async fn open_readonly<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI;
        debug!(
            "Opening sqlite db readonly with client: {}",
            path.as_ref().display()
        );
        SqliteDbAsyncClient::open(path, Some(flags)).await
    }

    pub async fn open_existing<P: AsRef<Path>>(
        path: P,
        flags: Option<OpenFlags>,
    ) -> SqliteResult<Self> {
        debug!(
            "Opening sqlite db existing with client: {}",
            path.as_ref().display()
        );
        if !file_exists(&path).await {
            return Err(SqliteError::FileDoesNotExist(
                path.as_ref().display().to_string(),
            ));
        }
        SqliteDbAsyncClient::open(path, flags).await
    }
}
#[async_trait]
pub trait AsyncSqliteConn: Send + Sync {
    async fn conn<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static;
}

#[async_trait]
pub trait AsyncSqliteConnMut: Send + Sync {
    async fn conn_mut<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static;
}

#[async_trait]
impl AsyncSqliteConn for SqliteDbAsyncClient {
    async fn conn<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn(func).await.map_err(Into::into)
    }
}

#[async_trait]
impl AsyncSqliteConnMut for SqliteDbAsyncClient {
    async fn conn_mut<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn_mut(func).await.map_err(Into::into)
    }
}

#[async_trait]
impl<T> Sqlike3Async for T
where
    T: AsyncSqliteConn + Debug,
{
    async fn analyze(&self) -> SqliteResult<usize> {
        self.conn(analyze).await.map_err(Into::into)
    }

    async fn is_empty_db(&self) -> SqliteResult<bool> {
        self.conn(is_empty_db).await.map_err(Into::into)
    }

    async fn pragma_index_list(
        &self,
        table: &str,
    ) -> SqliteResult<Vec<PragmaIndexListRow>> {
        let table = table.to_string();
        self.conn(move |conn| pragma_index_list(conn, &table))
            .await
            .map_err(Into::into)
    }

    async fn pragma_page_count(&self) -> SqliteResult<i64> {
        self.conn(pragma_page_count).await.map_err(Into::into)
    }

    async fn pragma_freelist_count(&self) -> SqliteResult<i64> {
        self.conn(pragma_freelist_count).await.map_err(Into::into)
    }

    async fn pragma_page_size(&self) -> SqliteResult<i64> {
        self.conn(|conn| pragma_page_size(conn, None))
            .await
            .map_err(Into::into)
    }

    async fn pragma_page_size_set(&self, page_size: i64) -> SqliteResult<i64> {
        self.conn(move |conn| pragma_page_size_set(conn, page_size))
            .await
            .map_err(Into::into)
    }

    async fn pragma_table_list(&self) -> SqliteResult<Vec<PragmaTableListRow>> {
        self.conn(pragma_table_list).await.map_err(Into::into)
    }

    async fn pragma_analysis_limit(&self) -> SqliteResult<usize> {
        self.conn(analysis_limit).await.map_err(Into::into)
    }

    async fn vacuum(&self) -> SqliteResult<usize> {
        self.conn(vacuum).await.map_err(Into::into)
    }

    async fn vacuum_into(&self, dst: String) -> SqliteResult<usize> {
        let dst = dst.to_string();
        self.conn(move |conn| vacuum_into(conn, dst))
            .await
            .map_err(Into::into)
    }

    async fn attach_db(&self, db: &str, as_: &str) -> SqliteResult<()> {
        let db = db.to_string();
        let as_ = as_.to_string();
        self.conn(move |conn| attach_db(conn, &db, &as_))
            .await
            .map_err(Into::into)
    }

    async fn detach_db(&self, db: &str) -> SqliteResult<()> {
        let db = db.to_string();
        self.conn(move |conn| detach_db(conn, &db))
            .await
            .map_err(Into::into)
    }
}
