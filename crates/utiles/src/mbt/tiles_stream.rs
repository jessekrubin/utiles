use rusqlite::Connection;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, warn};

use utiles_core::prelude::*;

use crate::mbt::MbtilesClientAsync;
use crate::sqlite::{AsyncSqliteConn, RusqliteResult};
use crate::tile_stream::TileReceiverStream;
use crate::UtilesResult;

pub fn make_tiles_rx(
    mbt: &MbtilesClientAsync,
    query_override: Option<&str>,
) -> UtilesResult<tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    tokio::spawn({
        // TODO: figure out if this is bad... or problematic...
        let mbt = mbt.clone();
        let query_override = query_override
            .unwrap_or(
                "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
            )
            .to_string();
        async move {
            let result = mbt
                .conn(move |c: &Connection| -> RusqliteResult<()> {
                    let mut s = c.prepare(&query_override)?;
                    let z_column = s.column_index("zoom_level")?;
                    let x_column = s.column_index("tile_column")?;
                    let y_column = s.column_index("tile_row")?;
                    let tile_data_column = s.column_index("tile_data")?;

                    let tiles_iters = s.query_map(rusqlite::params![], |row| {
                        let z: u8 = row.get(z_column)?;
                        let x: u32 = row.get(x_column)?;
                        let yup: u32 = row.get(y_column)?;
                        let tile = utile_yup!(x, yup, z);
                        let tile_data: Vec<u8> = row.get(tile_data_column)?;
                        let tx = tx.clone();
                        let tuple = (tile, tile_data);
                        if let Err(e) = tx.blocking_send(tuple) {
                            warn!("Blocking send error: {:?}", e);
                        }
                        Ok(())
                    })?;
                    // Consume the iterator
                    for row in tiles_iters {
                        let _ = row;
                    }

                    Ok(())
                })
                .await;
            if let Err(e) = result {
                error!("make_tiles_rx: {:?}", e);
            }
        }
    });
    Ok(rx)
}

pub fn make_tiles_stream(
    mbt: &MbtilesClientAsync,
    query_override: Option<&str>,
) -> UtilesResult<TileReceiverStream> {
    let rx = make_tiles_rx(mbt, query_override)?;
    Ok(ReceiverStream::new(rx))
}

impl MbtilesClientAsync {
    pub fn tiles_rx(
        &self,
        query_override: Option<&str>,
    ) -> UtilesResult<tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>> {
        make_tiles_rx(self, query_override)
    }
    pub fn tiles_stream(
        &self,
        query_override: Option<&str>,
    ) -> UtilesResult<TileReceiverStream> {
        let rx = self.tiles_rx(query_override)?;
        Ok(ReceiverStream::new(rx))
    }
}
