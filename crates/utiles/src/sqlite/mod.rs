//! Sqlite3 database util(e)s
//!
//! Pure sqlite database utils, helpers and bears oh my!
pub use rusqlite::{Connection, Result as RusqliteResult};

pub use affected::{AffectedType, RowsAffected};
pub use attach::{attach_db, detach_db};
pub use db::*;
pub use errors::{SqliteError, SqliteResult};
pub use insert_strategy::InsertStrategy;
pub use page_size::{is_valid_page_size, pragma_page_size_get};
pub use pragma::*;
pub use sqlike3::Sqlike3;

mod affected;
mod attach;
mod db;
mod errors;
mod insert_strategy;
mod page_size;
mod pragma;
mod sqlike3;

pub type AsyncSqliteResult<T> = Result<T, async_sqlite::Error>;
pub type AsyncSqliteError = async_sqlite::Error;
