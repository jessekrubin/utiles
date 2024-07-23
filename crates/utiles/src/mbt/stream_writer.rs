use futures::StreamExt;
use rusqlite::params_from_iter;
use rusqlite::types::Value;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, warn};

use utiles_core::{Tile, TileLike};

use crate::utilesqlite::Mbtiles;
use crate::UtilesResult;

#[derive(Default)]
pub struct MbtWriterStats {
    pub count: usize,
    pub nbytes: usize,
}

pub struct MbtStreamWriter {
    pub stream: ReceiverStream<(Tile, Vec<u8>)>,
    pub mbt: Mbtiles,
    pub stats: MbtWriterStats,
}

impl MbtStreamWriter {
    pub async fn write(&mut self) -> UtilesResult<()> {
        let mut stmt = self.mbt.conn.prepare(
            "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4);",
        )?;
        while let Some(value) = self.stream.next().await {
            let (tile, tile_data) = value;
            let tile_params = rusqlite::params![tile.z, tile.x, tile.y, tile_data];
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

    pub async fn write_batched(&mut self) -> UtilesResult<()> {
        let mut batch = vec![];
        while let Some(value) = self.stream.next().await {
            let (tile, tile_data) = value;
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
                &format!("INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES {};", placeholders),
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
        Ok(())
    }
}
