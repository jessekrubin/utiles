use futures::{Stream, stream, StreamExt};
use rusqlite::{Connection, Row};
use tokio::join;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};

use utiles_core::prelude::*;

use crate::cli::args::WebpifyArgs;
use crate::img::webpify_image;
use crate::sqlite::{AsyncSqliteConn, RusqliteResult};
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::{UtilesError, UtilesResult};


pub struct UtilesTilesProgress {
    pub count: usize,
}

pub async fn make_tiles_rx(
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<
    tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>,
> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    tokio::spawn({
        let mbt = mbt.clone();
        async move {
            mbt.conn(move |c: &Connection| -> RusqliteResult<()> {
                let mut s = c.prepare(
                    "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
                )?;
                let tiles_iters = s.query_map(rusqlite::params![], |row| {
                    let z: u8 = row.get(0)?;
                    let x: u32 = row.get(1)?;
                    let yup: u32 = row.get(2)?;
                    let tile = utile_yup!(x, yup, z);
                    let tile_data: Vec<u8> = row.get(3)?;
                    println!("sending tile: {:?}", tile);

                    let tx = tx.clone();
                    let tuple = (tile, tile_data);
                    if let Err(e) = tx.blocking_send(tuple) {
                        println!("send_res: {:?}", e);
                    }
                    Ok(())
                })?;
                // Consume the iterator
                for row in tiles_iters {
                    let _ = row;
                }

                Ok(())
            })
                .await
                .unwrap();
        }
    });
    Ok(rx)
}
pub async fn make_tiles_stream(
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<
    ReceiverStream<(Tile, Vec<u8>)>,
> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    tokio::spawn({
        let mbt = mbt.clone();
        async move {
            mbt.conn(move |c: &Connection| -> RusqliteResult<()> {
                let mut s = c.prepare(
                    "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
                )?;
                let tiles_iters = s.query_map(rusqlite::params![], |row| {
                    let z: u8 = row.get(0)?;
                    let x: u32 = row.get(1)?;
                    let yup: u32 = row.get(2)?;
                    let tile = utile_yup!(x, yup, z);
                    let tile_data: Vec<u8> = row.get(3)?;
                    println!("sending tile: {:?}", tile);

                    let tx = tx.clone();
                    let tuple = (tile, tile_data);
                    if let Err(e) = tx.blocking_send(tuple) {
                        println!("send_res: {:?}", e);
                    }
                    Ok(())
                })?;
                // Consume the iterator
                for row in tiles_iters {
                    let _ = row;
                }

                Ok(())
            })
                .await
                .unwrap();
        }
    });

    let tiles_stream = ReceiverStream::new(rx);
    Ok(tiles_stream)
}


pub struct WriterStats {
    pub count: usize,
    pub nbytes: usize,
}

impl Default for WriterStats {
    fn default() -> Self {
        Self {
            count: 0,
            nbytes: 0,
        }
    }
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
            let insert_res = stmt.execute(rusqlite::params![tile.z, tile.x, tile.y, tile_data]);
            let fake_res = 1usize;
            // self.stats.count += 1;
            // self.stats.nbytes += tile_data.len();
            // debug!("count: {}, nbytes: {}", self.stats.count, self.stats.nbytes);
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


// runs some function on each tile from a stream, and send to another stream
pub struct Worker {
    pub func: Box<dyn Fn(&(Tile, Vec<u8>)) -> UtilesResult<(Tile, Vec<u8>)> + Send + 'static>,
    pub out_tx: tokio::sync::mpsc::Sender<(Tile, Vec<u8>)>,
    pub rx: tokio::sync::mpsc::Receiver<(Tile, Vec<u8>)>,
    pub tx: tokio::sync::mpsc::Sender<(Tile, Vec<u8>)>,
}

unsafe impl Send for Worker {}

impl Worker {
    pub async fn run(&mut self) {
        // let mut stream = self.stream;
        // let mut tx = self.tx;
        // let mut out_rx = self.out_rx;
        // let func = &self.func;

        while let Some(value) = self.rx.recv().await {
            let (tile, tile_data) = value;
            let res = (self.func)(&(tile, tile_data));
            match res {
                Ok((tile, webp_bytes)) => {
                    if let Err(e) = self.out_tx.send((tile, webp_bytes)).await {
                        warn!("worker send_res: {:?}", e);
                    }
                }
                Err(e) => {
                    warn!("worker send_res: {:?}", e);
                }
            }
        }
        // tokio::spawn({
        //     // let mut stream = self.stream.clone();
        //     let tx = self.out_tx.clone();
        //     // let func = self.func.clone();
        //     async move {
        //         while let Some(value) = rx.recv().await {
        //             let (tile, tile_data) = value;
        //             let res = (self.func)(&(tile, tile_data));
        //             match res {
        //                 Ok((tile, webp_bytes)) => {
        //                     if let Err(e) = tx.send((tile, webp_bytes)).await {
        //                         warn!("worker send_res: {:?}", e);
        //                     }
        //                 }
        //                 Err(e) => {
        //                     warn!("worker send_res: {:?}", e);
        //                 }
        //             }
        //         }
        //     }
        //
        // });

        // tokio::spawn(async move {
        //     while let Some(value) = self.stream.next().await {
        //         let (tile, tile_data) = value;
        //
        //         let res = (self.func)(&(tile, tile_data));
        //         match res {
        //             Ok((tile, webp_bytes)) => {
        //                 self.out_rx.send((tile, webp_bytes)).await;
        //             }
        //             Err(e) => {
        //                 warn!("worker send_res: {:?}", e);
        //             }
        //         }
        //     }
        // });
    }
}

pub struct WorkerPool {
    pub stream: ReceiverStream<(Tile, Vec<u8>)>,
    pub workers: Vec<Worker>,
}

impl WorkerPool {
    pub fn new(stream: ReceiverStream<(Tile, Vec<u8>)>, workers: Vec<Worker>) -> Self {
        Self {
            stream,
            workers,
        }
    }

    pub async fn run(&mut self) -> UtilesResult<()> {
        let mut cur_worker = 0;
        let num_workers = self.workers.len();

        // let mut worker_futs = vec![];
        // // start each worker
        // for mut worker in &self.workers {
        //     let worker_fut = tokio::spawn({
        //         // let mut worker = worker.clone();
        //         async move {
        //             worker.run().await;
        //         }
        //     });
        //     worker_futs.push(worker_fut);
        // }
        //
        // start workers


        while let Some(value) = self.stream.next().await {
            let (tile, tile_data) = value;
            let worker = &self.workers[cur_worker];
            worker.tx.send((tile, tile_data)).await;
            cur_worker = (cur_worker + 1) % num_workers;
        }
        Ok(())
    }
}


fn worker_noop(
    (tile, tile_data): &(Tile, Vec<u8>),
) -> UtilesResult<(Tile, Vec<u8>)> {
    Ok((tile.clone(), tile_data.clone()))
}

pub async fn webpify_main(args: WebpifyArgs) -> UtilesResult<()> {
    info!("WEBPIFY");
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let mbt_metadata = mbt.metadata_rows().await?;
    let dst_mbtiles = Mbtiles::open_new(args.dst, None)?;
    dst_mbtiles.metadata_set_many(&mbt_metadata)?;
    dst_mbtiles.metadata_set("format", "webp")?;
    let tiles_stream = make_tiles_stream(&mbt).await?;

    let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);

    // let (tx_worker, rx_worker) = tokio::sync::mpsc::channel(100);

    // let one_worker = Worker {
    //     // stream: ReceiverStream::new(rx_worker),
    //     func: Box::new(
    //         |(tile, tile_data)| -> UtilesResult<(Tile, Vec<u8>)> {
    //             let webp_bytes = webpify_image(&tile_data)?;
    //             Ok((*tile, webp_bytes))
    //         },
    //     ),
    //     out_tx: tx_writer.clone(),
    //     rx: rx_worker,
    //     tx: tx_worker,
    // };
    //
    // let mut worker_pool = WorkerPool {
    //     stream: tiles_stream,
    //     workers: vec![one_worker],
    // };

    // use for_each_concurrent
    // let worker_pool = tokio::spawn(async move {
    //     tiles_stream
    //         .for_each_concurrent(1, |(tile, tile_data)| async move {
    //             let webp_bytes = webpify_image(&tile_data)?;
    //             tx_writer.send((tile, webp_bytes)).await?;
    //             Ok(())
    //         })
    //         .await;
    //     Ok(())
    // });

    let start_time = std::time::Instant::now();
    let mut writer = MbtStreamWriter {
        stream: ReceiverStream::new(rx_writer),
        mbt: dst_mbtiles,
        stats: WriterStats::default(),
    };
    let proc_future = tokio::spawn(async move {
        tiles_stream.for_each_concurrent(4, |(tile, tile_data)| {
            let tx_writer = tx_writer.clone();
            async move {
                println!(
                    "{}", tile.to_string()
                );

                // do it on blocking thread
                let blocking_res = tokio::task::spawn_blocking(move || {
                    let webpify_result = webpify_image(&tile_data);
                    webpify_result
                }).await;
                match blocking_res {
                    Err(je) => {
                        warn!("join-error: {:?}", je);
                    }
                    Ok(webpify_result) => {
                        match webpify_result {
                            Ok(webp_bytes) => {
                                let send_res =
                                    tx_writer.send((tile, webp_bytes)).await;
                                if let Err(e) = send_res {
                                    warn!("send_res: {:?}", e);
                                }
                            }
                            Err(e) => {
                                warn!("webpify_image: {:?}", e);
                            }
                        }
                    }

                }
                // match blocking_res {
                //     Ok(webp_bytes) => {
                //         let send_res =
                //             tx_writer.send((tile, webp_bytes)).await;
                //         if let Err(e) = send_res {
                //             warn!("send_res: {:?}", e);
                //         }
                //     }
                //     Err(e) => {
                //         warn!("webpify_image: {:?}", e);
                //     }
                // }
                // match webpify_result {
                //     Ok(webp_bytes) => {
                //         let send_res =
                //             tx_writer.send((tile, webp_bytes)).await;
                //         if let Err(e) = send_res {
                //             warn!("send_res: {:?}", e);
                //         }
                //     }
                //     Err(e) => {
                //         warn!("webpify_image: {:?}", e);
                //     }
                // }

                // tx_writer.send((tile, webp_bytes)).await?;
                // Ok(())
            }
        }).await
    });



    let results = join!(
        proc_future,
        writer.write()
    );
    let elapsed = start_time.elapsed();
    info!("elapsed: {:?}", elapsed);
    // join!(proc_future, writer.write());
    Ok(())
}


////////////////////
// pub async fn make_tiles_stream2(
//     mbt: MbtilesAsyncSqliteClient,
// ) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>> {
//     let (tx, rx) = tokio::sync::mpsc::channel(100);
//
//     // Here we create a stream of tiles from the source mbtiles,
//     // and then we process each tile and send progress updates
//     let thingy_of_tiles = tokio::spawn(async move {
//         mbt.conn(move |c: &Connection| -> RusqliteResult<()> {
//             let mut s = c.prepare(
//                 "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
//             )?;
//
//             let thingy = |row: &Row| {
//                 let z: u8 = row.get(0)?;
//                 let x: u32 = row.get(1)?;
//                 let yup: u32 = row.get(2)?;
//                 let tile = utile_yup!(x, yup, z);
//                 let tile_data: Vec<u8> = row.get(3)?;
//                 // Clone tx to avoid moving it into closure
//                 // let tx = tx.clone();
//                 let tuple = (tile, tile_data);
//                 if let Err(e) = tx.blocking_send(tuple) {
//                     warn!("send_res: {:?}", e);
//                     // println!("send_res: {:?}", e);
//                 }
//                 Ok(())
//             };
//             let tiles_iters = s.query_map(
//                 rusqlite::params![],
//                 thingy,
//                 // |row| {
//                 //     let z: u8 = row.get(0)?;
//                 //     let x: u32 = row.get(1)?;
//                 //     let yup: u32 = row.get(2)?;
//                 //     let tile = utile_yup!(x, yup, z);
//                 //     let tile_data: Vec<u8> = row.get(3)?;
//                 //     // Clone tx to avoid moving it into closure
//                 //     // let tx = tx.clone();
//                 //     let tuple = (tile, tile_data);
//                 //     if let Err(e) = tx.blocking_send(tuple) {
//                 //         warn!("send_res: {:?}", e);
//                 //         // println!("send_res: {:?}", e);
//                 //     }
//                 //     Ok(())
//                 // },
//             )?;
//
//             // Consume the iterator
//             for row in tiles_iters {
//                 let _ = row;
//             }
//
//             Ok(())
//         })
//             .await
//             .unwrap();
//     });
//
//     // Start the tile streaming task
//     tokio::spawn(async move {
//         let _ = thingy_of_tiles.await;
//     });
//
//     let tiles_stream = ReceiverStream::new(rx);
//     Ok(tiles_stream)
// }
