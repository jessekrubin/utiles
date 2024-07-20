use futures::StreamExt;
use rusqlite::Connection;
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use utiles_core::prelude::*;

use crate::cli::args::WebpifyArgs;
use crate::img::webpify_image;
use crate::sqlite::{AsyncSqliteConn, RusqliteResult};
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

pub fn make_tiles_rx(
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    tokio::spawn({
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
                        // println!("sending tile: {:?}", tile);

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

#[derive(Default)]
pub struct WriterStats {
    pub count: usize,
    pub nbytes: usize,
}

pub struct MbtStreamWriter {
    pub stream: ReceiverStream<(Tile, Vec<u8>)>,
    pub mbt: Mbtiles,
    pub stats: WriterStats,
}

impl MbtStreamWriter {
    // pub fn new(rx:
    //            tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>
    //            , conn: Mbtiles) -> Self {
    //     Self {
    //         rx,
    //         mbt: conn,
    //         stats: WriterStats {
    //             count: 0,
    //         },
    //     }
    // }

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

pub async fn webpify_main(args: WebpifyArgs) -> UtilesResult<()> {
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    dst_mbtiles.metadata_set("format", "webp")?;
    let tiles_stream = make_tiles_stream(&mbt)?;

    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriter {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: WriterStats::default(),
    };
    let proc_future = tokio::spawn(async move {
        // TODO: cli flag for concurrency
        tiles_stream
            .for_each_concurrent(4, |(tile, tile_data)| {
                let tx_writer = tx_writer.clone();
                async move {
                    let blocking_res =
                        tokio::task::spawn_blocking(move || webpify_image(&tile_data))
                            .await;
                    match blocking_res {
                        Err(je) => {
                            warn!("join-error: {:?}", je);
                        }
                        Ok(webpify_result) => match webpify_result {
                            Ok(webp_bytes) => {
                                let send_res = tx_writer.send((tile, webp_bytes)).await;
                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                }
                            }
                            Err(e) => {
                                warn!("webpify_image: {:?}", e);
                            }
                        },
                    }
                }
            })
            .await;
    });

    let (result, writer_result) = join!(proc_future, writer.write());
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    result?;
    writer_result?;
    // join!(proc_future, writer.write());
    Ok(())
}
