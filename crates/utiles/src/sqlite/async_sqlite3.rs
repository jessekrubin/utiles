use std::fmt::Debug;

use async_sqlite::{Client, Error as AsyncSqliteError};
use async_trait::async_trait;
use rusqlite::Connection;

use crate::sqlite::{analyze, attach_db, detach_db, is_empty_db, pragma_freelist_count, pragma_index_list, pragma_page_count, pragma_page_size, pragma_page_size_set, pragma_table_list, PragmaIndexListRow, PragmaTableListRow, SqliteResult, vacuum, vacuum_into};
use crate::sqlite::sqlike3::Sqlike3Async;
use crate::utilesqlite::mbtiles_async_sqlite::AsyncSqlite;

pub struct SqliteDbAsyncClient {
    pub client: Client,
}

#[async_trait]
pub trait AsyncSqliteConn: Send + Sync {
    async fn conn<F, T>(&self, func: F) -> Result<T, AsyncSqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static;
}

#[async_trait]
impl AsyncSqliteConn for SqliteDbAsyncClient {
    async fn conn<F, T>(&self, func: F) -> Result<T, AsyncSqliteError>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.client.conn(func).await
    }
}

#[async_trait]
impl<T> Sqlike3Async for T
where
    T: AsyncSqliteConn + Debug,
{
    async fn analyze(&self) -> SqliteResult<usize> {
        self.conn(|conn| analyze(conn)).await.map_err(Into::into)
    }


    async fn is_empty_db(&self) -> SqliteResult<bool> {
        self.conn(|conn| is_empty_db(conn)).await.map_err(Into::into)
    }

    async fn pragma_index_list(&self, table: &str) -> SqliteResult<Vec<PragmaIndexListRow>> {
        let table = table.to_string();
        self.conn(move |conn| pragma_index_list(conn, &table)).await.map_err(Into::into)
    }

    async fn pragma_page_count(&self) -> SqliteResult<i64> {
        self.conn(|conn| pragma_page_count(conn)).await.map_err(Into::into)
    }

    async fn pragma_freelist_count(&self) -> SqliteResult<i64> {
        self.conn(|conn| pragma_freelist_count(conn)).await.map_err(Into::into)
    }

    async fn pragma_page_size(&self) -> SqliteResult<i64> {
        self.conn(|conn| pragma_page_size(conn, None)).await.map_err(Into::into)
    }

    async fn pragma_page_size_set(&self, page_size: i64) -> SqliteResult<i64> {
        let page_size = page_size;
        self.conn(move |conn| pragma_page_size_set(conn, page_size)).await.map_err(Into::into)
    }

    async fn pragma_table_list(&self) -> SqliteResult<Vec<PragmaTableListRow>> {
        self.conn(|conn| pragma_table_list(conn)).await.map_err(Into::into)
    }

    async fn vacuum(&self) -> SqliteResult<usize> {
        self.conn(|conn| vacuum(conn)).await.map_err(Into::into)
    }

    async fn vacuum_into(&self, dst: String) -> SqliteResult<usize> {
        let dst = dst.to_string();
        self.conn(move |conn| vacuum_into(conn, dst)).await.map_err(Into::into)
    }


    async fn attach_db(&self, db: &str, as_: &str) -> SqliteResult<()> {
        let db = db.to_string();
        let as_ = as_.to_string();
        self.conn(move |conn| attach_db(conn, &db, &as_)).await.map_err(Into::into)
    }

    async fn detach_db(&self, db: &str) -> SqliteResult<()> {
        let db = db.to_string();
        self.conn(move |conn| detach_db(conn, &db)).await.map_err(Into::into)
    }
}