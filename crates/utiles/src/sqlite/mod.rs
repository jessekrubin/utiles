//! Sqlite3 database util(e)s
//!
//! Pure sqlite database utils, helpers and bears oh my!
pub use rusqlite::{Connection, Result as RusqliteResult};

pub use affected::{AffectedType, RowsAffected};
pub use async_sqlite3::{AsyncSqliteConn, AsyncSqliteConnMut, SqliteDbAsyncClient};
pub use attach::{attach_db, detach_db};
pub use db::*;
pub use dbpath::*;
pub use errors::{SqliteError, SqliteResult};
pub use header::*;
pub use insert_strategy::InsertStrategy;
pub use page_size::is_valid_page_size;
pub use pragma::*;
pub use sqlike3::{Sqlike3, Sqlike3Async};
mod affected;
mod async_sqlite3;
mod attach;
mod db;
mod dbpath;
mod errors;

pub mod header;
mod insert_strategy;
mod page_size;
mod pragma;
mod sqlike3;
pub mod streams;

pub type AsyncSqliteResult<T> = Result<T, async_sqlite::Error>;
pub type AsyncSqliteError = async_sqlite::Error;
