use futures::StreamExt;
use rusqlite::params_from_iter;
use rusqlite::types::Value;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, warn};

use utiles_core::{Tile, TileLike};

use crate::hash::xxh64_be_hex_upper;
use crate::mbt::{MbtType, Mbtiles};
use crate::{UtilesError, UtilesResult};

#[derive(Default)]
pub struct MbtWriterStats {
    pub count: usize,
    pub nbytes: usize,
}

pub struct MbtStreamWriterSync {
    pub stream: ReceiverStream<(Tile, Vec<u8>, Option<String>)>,
    pub mbt: Mbtiles,
    pub stats: MbtWriterStats,
}

impl MbtStreamWriterSync {
    pub fn preflight(&self) -> UtilesResult<()> {
        self.mbt
            .conn
            .execute_batch(
                r"
            PRAGMA synchronous = OFF;
            PRAGMA journal_mode = WAL;
            PRAGMA locking_mode = EXCLUSIVE;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = 100000;
            ",
            )
            .map_err(Into::into)
    }

    pub fn postflight(&self) -> UtilesResult<()> {
        self.mbt
            .conn
            .execute_batch(
                r"
            PRAGMA synchronous = NORMAL;
            PRAGMA journal_mode = DELETE;
            PRAGMA locking_mode = NORMAL;
            PRAGMA temp_store = DEFAULT;
            PRAGMA cache_size = 2000;
            ",
            )
            .map_err(Into::into)
    }

    pub async fn write_flat(&mut self) -> UtilesResult<()> {
        let mut stmt = self.mbt.conn.prepare(
            "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4);",
        )?;
        let stream = &mut self.stream;
        while let Some(value) = stream.next().await {
            let (tile, tile_data, _hash) = value;
            let tile_params = rusqlite::params![tile.z, tile.x, tile.yup(), tile_data];
            let insert_res = stmt.execute(tile_params);
            if let Err(e) = insert_res {
                warn!("insert_res: {:?}", e);
            } else {
                self.stats.count += 1;
                self.stats.nbytes += tile_data.len();
                debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
            }
        }
        Ok(())
    }

    pub async fn write_hash(&mut self) -> UtilesResult<()> {
        let mut stmt = self.mbt.conn.prepare(
            "INSERT INTO tiles_with_hash (zoom_level, tile_column, tile_row, tile_data, tile_hash) VALUES (?1, ?2, ?3, ?4, ?5);",
        )?;
        let stream = &mut self.stream;
        while let Some(value) = stream.next().await {
            let (tile, tile_data, hash_hex) = value;
            let hash_hex = hash_hex.unwrap_or_else(|| xxh64_be_hex_upper(&tile_data));
            // let hash_hex = xxh64_be_hex_upper(&tile_data);
            let tile_params =
                rusqlite::params![tile.z, tile.x, tile.yup(), tile_data, hash_hex];
            let insert_res = stmt.execute(tile_params);
            if let Err(e) = insert_res {
                warn!("insert_res: {:?}", e);
            } else {
                self.stats.count += 1;
                self.stats.nbytes += tile_data.len();
                debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
            }
        }
        Ok(())
    }

    pub async fn write_norm(&mut self) -> UtilesResult<()> {
        let mut map_stmt = self.mbt.conn.prepare(
            "INSERT INTO map (zoom_level, tile_column, tile_row, tile_id) VALUES (?1, ?2, ?3, ?4);",
        )?;
        let mut blob_stmt = self.mbt.conn.prepare(
            "INSERT OR IGNORE INTO images (tile_id, tile_data) VALUES (?1, ?2);",
        )?;
        while let Some(value) = self.stream.next().await {
            let (tile, tile_data, hash_hex) = value;
            let hash_hex = hash_hex.unwrap_or_else(|| xxh64_be_hex_upper(&tile_data));
            let map_insert_res =
                rusqlite::params![tile.z, tile.x, tile.yup(), hash_hex];
            let map_insert_res = map_stmt.execute(map_insert_res);
            if let Err(e) = map_insert_res {
                warn!("insert_res: {:?}", e);
            } else {
                self.stats.count += 1;
                self.stats.nbytes += tile_data.len();
                debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
            }
            let blob_params = rusqlite::params![hash_hex, tile_data];
            let insert_res = blob_stmt.execute(blob_params);
            if let Err(e) = insert_res {
                warn!("blob insert res: {:?}", e);
            } else {
                self.stats.count += 1;
                self.stats.nbytes += tile_data.len();
                debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
            }
        }
        Ok(())
    }

    pub async fn write(&mut self) -> UtilesResult<()> {
        let db_type = self.mbt.query_mbt_type()?;
        self.preflight()?;
        // let db_type = MbtType::Flat;
        match db_type {
            MbtType::Flat => self.write_flat().await,
            MbtType::Hash => self.write_hash().await,
            MbtType::Norm => self.write_norm().await,
            _ => Err(UtilesError::Unsupported(
                "stream write for unknown db type".to_string(),
            )),
        }?;
        self.postflight()?;
        Ok(())
    }

    pub async fn write_batched(&mut self) -> UtilesResult<()> {
        self.preflight()?;
        let mut batch = vec![];
        while let Some(value) = self.stream.next().await {
            let (tile, tile_data, _hash) = value;
            self.stats.count += 1;
            self.stats.nbytes += tile_data.len();
            batch.push((tile, tile_data));
            // let insert_res =
            //     stmt.execute(rusqlite::params![tile.z, tile.x, tile.y, tile_data]);
            if batch.len() >= 100 {
                let placeholders = batch
                    .iter()
                    .map(|_| "(?, ?, ?, ?)")
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut stmt = self.mbt.conn.prepare_cached(
                    &format!("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES {placeholders};"),
                )?;
                // let mut param_values: Vec<Value> = vec![];
                let param_values: Vec<Value> = batch
                    .iter()
                    .flat_map(|(tile, tile_data)| {
                        vec![
                            Value::Integer(i64::from(tile.z())),
                            Value::Integer(i64::from(tile.x())),
                            Value::Integer(i64::from(tile.yup())),
                            Value::Blob(tile_data.clone()),
                        ]
                    })
                    .collect();
                let insert_res = stmt.execute(params_from_iter(param_values.iter()));
                batch.clear();
                if let Err(e) = insert_res {
                    warn!("insert_res: {:?}", e);
                } else {
                    debug!(
                        "count: {}, nbytes: {}",
                        self.stats.count, self.stats.nbytes
                    );
                }
            }
        }

        if !batch.is_empty() {
            let placeholders = batch
                .iter()
                .map(|_| "(?, ?, ?, ?)")
                .collect::<Vec<_>>()
                .join(", ");
            let mut stmt = self.mbt.conn.prepare_cached(
                &format!("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES {placeholders};"),
            )?;
            let param_values: Vec<Value> = batch
                .iter()
                .flat_map(|(tile, tile_data)| {
                    vec![
                        Value::Integer(i64::from(tile.z())),
                        Value::Integer(i64::from(tile.x())),
                        Value::Integer(i64::from(tile.yup())),
                        Value::Blob(tile_data.clone()),
                    ]
                })
                .collect();
            let insert_res = stmt.execute(params_from_iter(param_values.iter()));
            if let Err(e) = insert_res {
                warn!("insert_res: {:?}", e);
            } else {
                debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
            }
        }
        self.postflight()?;
        Ok(())
    }
}
