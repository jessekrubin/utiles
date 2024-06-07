//! Sqlite3 database util(e)s
//!
//! Pure sqlite database utils, helpers and bears oh my!
pub use rusqlite::{Connection, Result as RusqliteResult};

pub use db::*;
pub use insert_strategy::InsertStrategy;
pub use pragma::*;
pub use sqlike3::Sqlike3;

mod db;
mod errors;
mod insert_strategy;
mod page_size;
mod pragma;
mod sqlike3;

pub use errors::{SqliteError, SqliteResult};
pub use page_size::{is_valid_page_size, pragma_page_size_get};

pub type AsyncSqliteResult<T> = Result<T, async_sqlite::Error>;
