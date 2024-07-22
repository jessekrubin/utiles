use rusqlite::Connection;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, warn};

use utiles_core::prelude::*;

use crate::sqlite::{AsyncSqliteConn, RusqliteResult};
use crate::utilesqlite::MbtilesAsyncSqliteClient;
use crate::UtilesResult;

pub fn make_tiles_rx(
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    tokio::spawn({
        // TODO: figure out if this is bad... or problematic...
        let mbt = mbt.clone();
        async move {
            let result = mbt
                .conn(move |c: &Connection| -> RusqliteResult<()> {
                    let mut s = c.prepare(
                        "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
                    )?;
                    let tiles_iters = s.query_map(rusqlite::params![], |row| {
                        let z: u8 = row.get(0)?;
                        let x: u32 = row.get(1)?;
                        let yup: u32 = row.get(2)?;
                        let tile = utile_yup!(x, yup, z);
                        let tile_data: Vec<u8> = row.get(3)?;
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
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>> {
    let rx = make_tiles_rx(mbt)?;
    Ok(ReceiverStream::new(rx))
}
