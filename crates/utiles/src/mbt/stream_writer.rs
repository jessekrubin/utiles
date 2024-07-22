use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, warn};

use utiles_core::Tile;

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
            let insert_res =
                stmt.execute(rusqlite::params![tile.z, tile.x, tile.y, tile_data]);
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
}
