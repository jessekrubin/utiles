use std::io::Cursor;

use async_stream::stream;
use futures::{stream, StreamExt};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;
use rusqlite::{Connection, MappedRows, Row};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use utiles_core::prelude::*;
use utiles_core::{utile, utile_yup};

use crate::cli::args::WebpifyArgs;
use crate::sqlite::{AsyncSqliteConn, AsyncSqliteResult, RusqliteResult};
use crate::utilesqlite::mbtiles_async_sqlite;
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

fn webpify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
    let img = image::load_from_memory(&data)?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::WebP)?;
    Ok(buf)
}

fn pngify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
    let img = image::load_from_memory(&data)?;
    let mut buf = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut buf,
        CompressionType::Default,
        FilterType::Adaptive,
    );
    img.write_with_encoder(encoder)?;
    Ok(buf)
}

pub struct UtilesTilesProgress {
    pub count: usize,
}


// impl MbtilesAsyncSqliteClient {
//     pub async fn tiles_stream(&self) -> AsyncSqliteResult<impl Stream<Item = (Tile, Vec<u8>)>> {
//         let conn = self.conn()?;
//         let mut s = conn.prepare(
//             "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
//         )?;
//         let rows = s.query_map(rusqlite::params![], |row| {
//             let z: u8 = row.get(0)?;
//             let x: u32 = row.get(1)?;
//             let yup: u32 = row.get(2)?;
//             let tile = utile_yup!(x, yup, z);
//             let tile_data: Vec<u8> = row.get(3)?;
//             Ok((tile, tile_data))
//         })?;
//         Ok(stream::iter(rows))
//     }
// }
//
// use tokio::sync::mpsc::Sender;
// use rusqlite::{Connection, Result as RusqliteResult};
// use tokio_stream::wrappers::ReceiverStream;

pub async fn make_tiles_stream(
    mbt: &MbtilesAsyncSqliteClient,
) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>> {
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
            }).await.unwrap();
        }
    });

    let tiles_stream = ReceiverStream::new(rx);
    Ok(tiles_stream)
}

// pub async fn make_tiles_stream(
//     mbt: &MbtilesAsyncSqliteClient,
// ) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>> {
//     let (tx, rx) = tokio::sync::mpsc::channel(100);
//
//     let tx_clone = tx.clone();
//
//     let mbt = mbt.clone();
//     // Start the tile streaming task
//     let thingy_of_tiles = mbt.conn(move |c: &Connection| -> RusqliteResult<()> {
//         let mut s = c.prepare(
//             "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
//         )?;
//
//         let tiles_iters = s.query_map(rusqlite::params![], |row| {
//             let z: u8 = row.get(0)?;
//             let x: u32 = row.get(1)?;
//             let yup: u32 = row.get(2)?;
//             let tile = utile_yup!(x, yup, z);
//             let tile_data: Vec<u8> = row.get(3)?;
//             println!("sending tile: {:?}", tile);
//
//             // Clone tx to avoid moving it into closure
//             let tx_clone = tx_clone.clone();
//             let tuple = (tile, tile_data);
//             if let Err(e) = tx_clone.blocking_send(tuple) {
//                 println!("send_res: {:?}", e);
//             }
//             Ok(())
//         })?;
//
//         // Consume the iterator
//         for row in tiles_iters {
//             let _ = row;
//         }
//
//         Ok(())
//     });
//
//     tokio::spawn(async move {
//         if let Err(e) = thingy_of_tiles.await {
//             eprintln!("Error in tile streaming task: {:?}", e);
//         }
//     });
//
//     let tiles_stream = ReceiverStream::new(rx);
//     Ok(tiles_stream)
// }


pub async fn make_tiles_stream2(
    mbt: MbtilesAsyncSqliteClient,
) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Here we create a stream of tiles from the source mbtiles,
    // and then we process each tile and send progress updates
    let thingy_of_tiles = tokio::spawn(async move {
        mbt.conn(move |c: &Connection| -> RusqliteResult<()> {
            let mut s = c.prepare(
                "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
            )?;

            let thingy = |row: &Row| {
                let z: u8 = row.get(0)?;
                let x: u32 = row.get(1)?;
                let yup: u32 = row.get(2)?;
                let tile = utile_yup!(x, yup, z);
                let tile_data: Vec<u8> = row.get(3)?;
                // Clone tx to avoid moving it into closure
                // let tx = tx.clone();
                let tuple = (tile, tile_data);
                if let Err(e) = tx.blocking_send(tuple) {
                    warn!("send_res: {:?}", e);
                    // println!("send_res: {:?}", e);
                }
                Ok(())
            };
            let tiles_iters = s.query_map(rusqlite::params![],
            thingy,
                                          // |row| {
                                          //     let z: u8 = row.get(0)?;
                                          //     let x: u32 = row.get(1)?;
                                          //     let yup: u32 = row.get(2)?;
                                          //     let tile = utile_yup!(x, yup, z);
                                          //     let tile_data: Vec<u8> = row.get(3)?;
                                          //     // Clone tx to avoid moving it into closure
                                          //     // let tx = tx.clone();
                                          //     let tuple = (tile, tile_data);
                                          //     if let Err(e) = tx.blocking_send(tuple) {
                                          //         warn!("send_res: {:?}", e);
                                          //         // println!("send_res: {:?}", e);
                                          //     }
                                          //     Ok(())
                                          // },
            )?;

            // Consume the iterator
            for row in tiles_iters {
                let _ = row;
            }

            Ok(())
        }).await.unwrap();
    });

    // Start the tile streaming task
    tokio::spawn(async move {
        let _ = thingy_of_tiles.await;
    });

    let tiles_stream = ReceiverStream::new(rx);
    Ok(tiles_stream)
}

// pub async fn make_tiles_stream2(
//     mbt: &MbtilesAsyncSqliteClient,
// ) -> UtilesResult<ReceiverStream<(Tile, Vec<u8>)>>
// {
//     let (tx, rx) = tokio::sync::mpsc::channel(100);
//     // here we create a stream of tiles from the source mbtiles,
//     // and then we process each tile and send progress updates
//     let mbt = mbt.clone();
//     let thingy_of_tiles = mbt.conn(
//             move |c: &Connection| -> RusqliteResult<()>
//             {
//                 let tx = tx.clone();
//                 let mut s = c.prepare(
//                     "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
//                 )?;
//
//                 let tiles_iters = s.query_map(rusqlite::params![], |row| {
//                     let z: u8 = row.get(0)?;
//                     let x: u32 = row.get(1)?;
//                     let yup: u32 = row.get(2)?;
//                     let tile = utile_yup!(x, yup, z);
//                     let tile_data: Vec<u8> = row.get(3)?;
//                     println!("sending tile: {:?}", tile);
//                     let tuple = (tile, tile_data);
//                     let send_res = tx.blocking_send(
//                         tuple
//                     );
//                     if let Err(e) = send_res {
//                         println!("send_res: {:?}", e);
//                     };
//                     Ok(())
//                     // Ok(tuple)
//                 })?;
//
//                 // consume the iterator
//                 for row in tiles_iters {
//                     let _ = row;
//                 }
//
//
//                 Ok(())
//             }
//         );
//     // start the tile streaming task
//     tokio::spawn(
//         async move {
//             let _ = thingy_of_tiles.await;
//         }
//     );
//     let tiles_stream = ReceiverStream::new(rx);
//     Ok(tiles_stream)
// }

pub async fn webpify_main(args: WebpifyArgs) -> UtilesResult<()> {
    info!("WEBPIFY");
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;
    let mut dst_mbtiles = Mbtiles::open_new(args.dst, None)?;

    // let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // let mut tiles_stream = ReceiverStream::new(rx);

    // listener task
    // let task = tokio::spawn(async move {
    //     let mut count = 0;
    //     while let Some(progress) = rx.recv().await {
    //         count += 1;
    //         println!("progress: {:?}", progress);
    //         println!("count: {}", count);
    //         dst_mbtiles
    //     }
    // });

    let mut tiles_stream = make_tiles_stream(&mbt).await?;
    let mut count = 0;
    while let Some(value) = tiles_stream.next().await {
        count += 1;
        println!("count: {}", count);

        let (tile, tile_data) = value;
        println!("{tile:?}");

        let webp_bytes = webpify_image(&tile_data)?;
        dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
    };


    // let thingy_of_tiles =
    //
    //     // here we create a stream of tiles from the source mbtiles,
    //     // and then we process each tile and send progress updates
    //     mbt.conn(
    //         move |c: &Connection| -> RusqliteResult<()>
    //         {
    //             let tx = tx.clone();
    //             let mut s = c.prepare(
    //                 "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
    //             )?;
    //
    //             let tiles_iters = s.query_map(rusqlite::params![], |row| {
    //                 let z: u8 = row.get(0)?;
    //                 let x: u32 = row.get(1)?;
    //                 let yup: u32 = row.get(2)?;
    //                 let tile = utile_yup!(x, yup, z);
    //                 let tile_data: Vec<u8> = row.get(3)?;
    //                 println!("sending tile: {:?}", tile);
    //                 let tuple = (tile, tile_data);
    //                 let send_res = tx.blocking_send(
    //                     tuple
    //                 );
    //                 if let Err(e) = send_res {
    //                     println!("send_res: {:?}", e);
    //                 };
    //                 Ok(())
    //                 // Ok(tuple)
    //             })?;
    //
    //             // consume the iterator
    //             for row in tiles_iters {
    //                 let _ = row;
    //             }
    //
    //             //
    //             // // let tiles_stream = stream::iter(tiles_iters);
    //             // // // buffered stream
    //             // // let maped_rows = tiles_stream.buffered(10);
    //             // for row in tiles_iters {
    //             //     let (tile, tile_data) = row?;
    //             //     println!("{tile:?}");
    //             //     let send_res = tx.blocking_send(
    //             //         (tile, tile_data)
    //             //     );
    //             //     if let Err(e) = send_res {
    //             //         println!("send_res: {:?}", e);
    //             //     }
    //             //
    //             //     // let webp_bytes = webpify_image(&tile_data)?;
    //             //     // dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
    //             // }
    //             // tiles_stream.for_each_concurrent(
    //             //     2, |(tile, tile_data)| async {
    //             //         let webp_bytes = webpify_image(&tile_data)?;
    //             //         dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
    //             //         Ok(())
    //             //     }
    //             // )
    //
    //             Ok(())
    //
    //
    //             // Ok(rows)
    //
    //             // let mut done: usize = 0;
    //             //
    //             // for row in rows {
    //             //     let (tile, tile_data) = row?;
    //             //     yield (tile, tile_data);
    //             // }
    //             // Ok(true)
    //         }
    //     );
    //
    //
    // while let Some(value) = tiles_stream.next().await {
    //     println!("value: {:?}", value);
    // };
    // let _ = task.await;
    // for row in maped_rows {
    //     let (tile, tile_data) = row?;
    //     println!("{tile:?}");
    //     // let webp_bytes = webpify_image(&tile_data)?;
    //     // dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
    // }

    //     let mut src_tiles_stream = stream! {
    // ;
    //         for row in maped_rows {
    //             let (tile, tile_data) = row?;
    //             yield (tile, tile_data);
    //         }
    //     };

    // for each tile in the stream print the x, y, z... just as a test
    // for (tile, tile_data) in src_tiles_stream {
    //     info!("{tile:?}");
    // }
    // while let Some(value) = src_tiles_stream.next().await {
    // let (tile, tile_data) = value;
    // info!("{value:?}");
    // }
    //
    //
    // let thingy = mbt
    //     .conn(move |c: &Connection| -> RusqliteResult<bool> {
    //         let mut s = c.prepare(
    //             "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
    //         )?;
    //
    //         let mut rows = s.query_map(rusqlite::params![], |row| {
    //             let z: u8 = row.get(0)?;
    //             let x: u32 = row.get(1)?;
    //             let yup: u32 = row.get(2)?;
    //             let tile = utile_yup!(x, yup, z);
    //             // println!("{tile:?}");
    //             let tile_data: Vec<u8> = row.get(3)?;
    //             Ok((tile, tile_data))
    //             // Ok((z, x, y, tile_data))
    //         })?;
    //
    //         let mut done: usize = 0;
    //
    //         for row in rows {
    //             let (tile, tile_data) = row?;
    //             // println!("{tile:?}");
    //             // let webp_bytes = webpify_image(&tile_data).map_err(
    //             //     |e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))
    //             // )?;
    //             let webp_bytes = webpify_image(&tile_data).map_err(|e| {
    //                 rusqlite::Error::ToSqlConversionFailure(Box::new(e))
    //             })?;
    //             dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
    //             done += 1;
    //             tx.blocking_send(UtilesTilesProgress { count: done });
    //             if done % 1000 == 0 {
    //                 info!("done: {}", done);
    //             }
    //         }
    //         Ok(true)
    //     })
    //     .await?;
    //
    // // progress consumer task
    //
    //
    // while let Some(progress) =  rx.recv().await {
    //     info!("progress: {}", progress.count);
    // }


    Ok(())
}
