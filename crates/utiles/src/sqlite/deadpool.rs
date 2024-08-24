//! deadpool integration...

use async_trait::async_trait;
use deadpool::managed::{self, RecycleError};
use deadpool_sync::SyncWrapper;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::sqlite::{AsyncSqliteConn, SqliteError};
pub use deadpool::managed::reexports::*;
pub use deadpool_sync::reexports::*;
pub use rusqlite;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
    /// Path to SQLite database file.
    pub path: PathBuf,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create a new [`Config`] with the given `path` of SQLite database file.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            pool: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .map_err(CreatePoolError::Config)?
            .runtime(runtime)
            .build()
            .map_err(CreatePoolError::Build)
    }
    pub fn builder(&self, runtime: Runtime) -> Result<PoolBuilder, ConfigError> {
        let manager = Manager::from_config(self, runtime);
        Ok(Pool::builder(manager)
            .config(self.get_pool_config())
            .runtime(runtime))
    }

    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// This error is returned if there is something wrong with the SQLite configuration.
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;
deadpool::managed_reexports!(
    "rusqlite",
    Manager,
    managed::Object<Manager>,
    rusqlite::Error,
    ConfigError
);

/// Type alias for [`Object`]
pub type Connection = Object;

/// [`Manager`] for creating and recycling SQLite [`Connection`]s.
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    config: Config,
    recycle_count: AtomicUsize,
    runtime: Runtime,
}

impl Manager {
    /// Creates a new [`Manager`] using the given [`Config`] backed by the
    /// specified [`Runtime`].
    #[must_use]
    pub fn from_config(config: &Config, runtime: Runtime) -> Self {
        Self {
            config: config.clone(),
            recycle_count: AtomicUsize::new(0),
            runtime,
        }
    }
}

impl managed::Manager for Manager {
    type Type = SyncWrapper<rusqlite::Connection>;
    type Error = rusqlite::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let path = self.config.path.clone();
        SyncWrapper::new(self.runtime, move || rusqlite::Connection::open(path)).await
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if conn.is_mutex_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".into(),
            ));
        }
        let recycle_count = self.recycle_count.fetch_add(1, Ordering::Relaxed);
        let n: usize = conn
            .interact(move |conn| {
                conn.query_row("SELECT $1", [recycle_count], |row| row.get(0))
            })
            .await
            .map_err(|e| RecycleError::message(format!("{}", e)))??;
        if n == recycle_count {
            Ok(())
        } else {
            Err(RecycleError::message("Recycle count mismatch"))
        }
    }
}

pub struct SqliteDeadpool {
    pool: Pool,
}

#[async_trait]
impl AsyncSqliteConn for SqliteDeadpool {
    async fn conn<F, T>(&self, func: F) -> Result<T, SqliteError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.pool.get().await.unwrap();
        let result = conn
            .interact(|conn| func(conn))
            .await
            .map_err(|e| SqliteError::from(e))?;
        result.map_err(SqliteError::from)
    }
}

//
// impl Sqlike3Async for SqliteDeadpool {
//     fn conn(&self) -> &rusqlite::Connection {
//         unimplemented!()
//     }
//
//     fn conn_mut(&mut self) -> &mut rusqlite::Connection {
//         unimplemented!()
//     }
// }
