use std::time::Instant;
// fn main() {
//     println!("utiles ~ dev");
// }
use futures::StreamExt;
use rusqlite::{Connection, Result as RusqliteResult, Row};
use tokio::sync::mpsc::{channel, Receiver};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};
use utiles::mbt::{
    MbtStreamWriterSync, MbtWriterStats, Mbtiles, MbtilesAsync, MbtilesClientAsync,
};
use utiles::sqlite::AsyncSqliteConn;
use utiles::UtilesResult;
use utiles_core::utile_yup;

/// Creates and returns an async `Receiver` of items derived from rows in the DB.
/// `T` is the custom output type.
/// `F` is a closure that maps a `Row` to `T`.
pub fn make_stream_rx<T, F>(
    mbt: &MbtilesClientAsync,
    query_override: Option<&str>,
    row_mapper: F,
) -> UtilesResult<Receiver<T>>
where
    // The row_mapper must be callable from inside a `spawn_blocking`.
    F: Fn(&Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
{
    // Create a channel for streaming out `T` items.
    let (tx, rx) = channel::<T>(100);

    // Pick the query string; fallback to your usual default.
    let query = query_override
        .unwrap_or("SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;")
        .to_string();

    // Clone the mbt handle for the spawned task.
    let mbt_clone = mbt.clone();

    tokio::spawn(async move {
        // Perform the DB connection + row iteration on the blocking thread (via `.conn()`).
        let result = mbt_clone
            .conn(move |conn: &Connection| -> RusqliteResult<()> {
                let mut stmt = conn.prepare(&query)?;
                // Map each DB row into T via `row_mapper`.
                let rows_iter = stmt.query_map([], |row| {
                    // Convert row -> T
                    let item = row_mapper(row)?;

                    // Send to the channel (blocking_send is fine in this context).
                    if let Err(e) = tx.blocking_send(item) {
                        warn!("channel send error: {:?}", e);
                    }

                    Ok(())
                })?;

                // consume all rows
                for row_result in rows_iter {
                    // You can optionally handle row-level errors here
                    if let Err(e) = row_result {
                        error!("row error: {:?}", e);
                    }
                }
                Ok(())
            })
            .await;

        if let Err(e) = result {
            error!("make_stream_rx: DB error: {:?}", e);
        }
    });

    Ok(rx)
}

// static function qery
const QUERY: &str = r#"
WITH parent AS (
  SELECT DISTINCT
    (zoom_level - 1) AS p_z,
    (tile_column / 2) AS p_x,
    (tile_row / 2)    AS p_y
  FROM tiles
  WHERE zoom_level > 0
)
SELECT
  parent.p_z          AS parent_z,
  parent.p_x          AS parent_x,
  parent.p_y          AS parent_y,

  child_tl.tile_data  AS top_left_data,
  child_tr.tile_data  AS top_right_data,
  child_bl.tile_data  AS bottom_left_data,
  child_br.tile_data  AS bottom_right_data

FROM parent
-- top-left child
LEFT JOIN tiles child_tl
  ON child_tl.zoom_level  = parent.p_z + 1
 AND child_tl.tile_column = parent.p_x * 2
 AND child_tl.tile_row    = parent.p_y * 2

-- top-right child
LEFT JOIN tiles child_tr
  ON child_tr.zoom_level  = parent.p_z + 1
 AND child_tr.tile_column = parent.p_x * 2 + 1
 AND child_tr.tile_row    = parent.p_y * 2

-- bottom-left child
LEFT JOIN tiles child_bl
  ON child_bl.zoom_level  = parent.p_z + 1
 AND child_bl.tile_column = parent.p_x * 2
 AND child_bl.tile_row    = parent.p_y * 2 + 1

-- bottom-right child
LEFT JOIN tiles child_br
  ON child_br.zoom_level  = parent.p_z + 1
 AND child_br.tile_column = parent.p_x * 2 + 1
 AND child_br.tile_row    = parent.p_y * 2 + 1

"#;
#[derive(Debug)]
struct FourTileRow {
    parent_z: u8,
    parent_x: u32,
    parent_y: u32,
    top_left: Option<Vec<u8>>,
    top_right: Option<Vec<u8>>,
    bottom_left: Option<Vec<u8>>,
    bottom_right: Option<Vec<u8>>,
}
/// This is just a placeholder for your custom "merge 4 child tiles" into one 512x512 tile.
fn merge_256_tiles_into_512(
    tl: Option<&[u8]>,
    tr: Option<&[u8]>,
    bl: Option<&[u8]>,
    br: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    // 1) decode each child tile if it exists
    // 2) create a new 512x512 buffer
    // 3) place each 256 tile in the correct quadrant
    // 4) encode the final as PNG/WebP/whatever
    // For example, you could use image crate, etc.
    // We'll just return a stub here:
    Ok(vec![]) // Return empty
}
fn map_four_tile_row(row: &rusqlite::Row) -> rusqlite::Result<FourTileRow> {
    Ok(FourTileRow {
        parent_z: row.get("parent_z")?,
        parent_x: row.get("parent_x")?,
        parent_y: row.get("parent_y")?,
        top_left: row.get("top_left_data")?,
        top_right: row.get("top_right_data")?,
        bottom_left: row.get("bottom_left_data")?,
        bottom_right: row.get("bottom_right_data")?,
    })
}

// async fn stream_four_tiles(
//     mbt: &MbtilesClientAsync,
//     query: &str,
// ) -> UtilesResult<Receiver<FourTileRow>> {
//     let (tx, rx) = channel(100);
//     let query_string = query.to_string();
//
//     tokio::spawn({
//         let mbt_clone = mbt.clone();
//         async move {
//             let result = mbt_clone
//                 .conn(move |c: &Connection| -> RusqliteResult<()> {
//                     let mut s = c.prepare(QUERY)?;
//                     // let z_column = s.column_index("zoom_level")?;
//                     // let x_column = s.column_index("tile_column")?;
//                     // let y_column = s.column_index("tile_row")?;
//                     let tx = tx.clone();
//                     let rows_iter = s.query_map(rusqlite::params![], |row| {
//                         let item = map_four_tile_row(row)?;
//                         if let Err(e) = tx.blocking_send(item) {
//                             warn!("send error: {:?}", e);
//                         }
//                         Ok(())
//                         // let z: u8 = row.get(z_column)?;
//                         // let x: u32 = row.get(x_column)?;
//                         // let yup: u32 = row.get(y_column)?;
//                         // let tile = utile_yup!(x, yup, z);
//                         // if let Err(e) = tx.blocking_send(tile) {
//                         //     debug!("Blocking send error: {:?}", e);
//                         //     Ok(false)
//                         // } else {
//                         //     Ok(true)
//                         // }
//                     })?;
//                     // Consume the iterator
//                     for row_result in rows_iter {
//                         if let Err(e) = row_result {
//                             error!("row error: {:?}", e);
//                         }
//                     }
//                     Ok(())
//                     //
//                     // for row in tiles_iters {
//                     //     let _ = row;
//                     //     match row {
//                     //         Ok(true) => {}
//                     //         Ok(false) => {
//                     //             break;
//                     //         }
//                     //         Err(e) => {
//                     //             error!("enum tiles iter error: {:?}", e);
//                     //             break;
//                     //         }
//                     //     }
//                     // }
//                     // Ok(())
//                 })
//                 .await;
//             if let Err(e) = result {
//                 error!("make_enumerate_rx: {:?}", e);
//             }
//         }
//         // let result = mbt_clone
//         //     .conn(move |conn: &Connection| -> rusqlite::Result<()> {
//         //         let mut stmt = conn.prepare(&query_string)?;
//         //         let rows_iter = stmt.query_map([], |row| {
//         //             let item = map_four_tile_row(row)?;
//         //             if let Err(e) = tx.blocking_send(item) {
//         //                 warn!("send error: {:?}", e);
//         //             }
//         //             Ok(())
//         //         })?;
//         //
//         //         for row_result in rows_iter {
//         //             if let Err(e) = row_result {
//         //                 error!("row error: {:?}", e);
//         //             }
//         //         }
//         //         Ok(())
//         //     })
//         //     .await;
//         //
//         // if let Err(e) = result {
//         //     error!("stream_four_tiles: DB error: {:?}", e);
//         // }
//     });
//
//     Ok(rx)
// }
//
pub fn make_stream_rx_mapper<T, F>(
    mbt: &MbtilesClientAsync,
    query: &str,
    row_mapper: F,
) -> UtilesResult<Receiver<T>>
where
    F: Fn(&Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let mbt_clone = mbt.clone();
    let query_string = query.to_string();

    tokio::spawn(async move {
        let result = mbt_clone
            .conn(move |conn: &Connection| -> RusqliteResult<()> {
                let mut stmt = conn.prepare(&query_string)?;
                let rows_iter = stmt.query_map([], |row| {
                    let item = row_mapper(row)?;
                    // send to channel
                    if let Err(e) = tx.blocking_send(item) {
                        tracing::warn!("send error: {:?}", e);
                    }
                    Ok(())
                })?;
                // consume
                for row_result in rows_iter {
                    if let Err(e) = row_result {
                        tracing::error!("row error: {:?}", e);
                    }
                }
                Ok(())
            })
            .await;

        if let Err(e) = result {
            tracing::error!("make_stream_rx: DB error: {:?}", e);
        }
    });

    Ok(rx)
}

pub async fn utiles_512ify() -> Result<(), Box<dyn std::error::Error>> {
    info!("utiles ~ 512ify");

    let src_mbtiles = "osm-test-z1.mbtiles";
    let dst_mbtiles = "osm-test-z1-512.mbtiles";

    // 1) Open source
    let mbt = MbtilesClientAsync::open_existing(src_mbtiles).await?;
    mbt.assert_mbtiles().await?;

    // 2) Open or create the destination MBTiles
    // let dst = Mbtiles::from(dst_mbtiles);
    // Possibly set metadata, e.g. format=png or something
    // dst.metadata_set("name", "512ified")?;
    // dst.metadata_set("format", "png")?;

    // 3) Build the multi-join query
    let join_query = r#"
        WITH parent AS (
          SELECT DISTINCT
            (zoom_level - 1) AS p_z,
            (tile_column / 2) AS p_x,
            (tile_row / 2)    AS p_y
          FROM tiles
          WHERE zoom_level > 0
        )
        SELECT
          parent.p_z          AS parent_z,
          parent.p_x          AS parent_x,
          parent.p_y          AS parent_y,
          child_tl.tile_data  AS top_left_data,
          child_tr.tile_data  AS top_right_data,
          child_bl.tile_data  AS bottom_left_data,
          child_br.tile_data  AS bottom_right_data
        FROM parent
        LEFT JOIN tiles child_tl
          ON child_tl.zoom_level  = parent.p_z + 1
         AND child_tl.tile_column = parent.p_x * 2
         AND child_tl.tile_row    = parent.p_y * 2
        LEFT JOIN tiles child_tr
          ON child_tr.zoom_level  = parent.p_z + 1
         AND child_tr.tile_column = parent.p_x * 2 + 1
         AND child_tr.tile_row    = parent.p_y * 2
        LEFT JOIN tiles child_bl
          ON child_bl.zoom_level  = parent.p_z + 1
         AND child_bl.tile_column = parent.p_x * 2
         AND child_bl.tile_row    = parent.p_y * 2 + 1
        LEFT JOIN tiles child_br
          ON child_br.zoom_level  = parent.p_z + 1
         AND child_br.tile_column = parent.p_x * 2 + 1
         AND child_br.tile_row    = parent.p_y * 2 + 1
    "#;

    // 4) Stream the rows

    let thingystream = make_stream_rx(&mbt, Some(join_query), map_four_tile_row)?;


    //  for now just print each thingy....
    let mut stream = ReceiverStream::new(thingystream);
    while let Some(thingy) = stream.next().await {
        println!("thingy: {:?}", thingy);
    }


    // let four_tile_rx = stream_four_tiles(&mbt, join_query).await?;
    //
    // // 5) Prepare the writer
    // let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
    // let mut writer = MbtStreamWriterSync {
    //     stream: tokio_stream::wrappers::ReceiverStream::new(rx_writer),
    //     mbt: dst,
    //     stats: MbtWriterStats::default(),
    // };
    //
    // let start_time = Instant::now();
    //
    // // 6) Spawn a task to consume that stream, combine the child tiles -> a 512 tile,
    // //    and send to the writer channel
    // let proc_handle = tokio::spawn(async move {
    //     let stream = tokio_stream::wrappers::ReceiverStream::new(four_tile_rx);
    //     // Run concurrently if you want to:
    //     stream
    //         .for_each_concurrent(4, |row_item| {
    //             async move {
    //
    //                 println!("row_item: {:?}", row_item);
    //             }
    //
    //             // let tx_writer = tx_writer.clone();
    //             // async move {
    //             //     let parent_tile =
    //             //         (row_item.parent_x, row_item.parent_y, row_item.parent_z);
    //             //     // Combine child tiles into one 512 tile
    //             //     let merged_res = tokio::task::spawn_blocking(move || {
    //             //         merge_256_tiles_into_512(
    //             //             row_item.top_left.as_deref(),
    //             //             row_item.top_right.as_deref(),
    //             //             row_item.bottom_left.as_deref(),
    //             //             row_item.bottom_right.as_deref(),
    //             //         )
    //             //     })
    //             //     .await;
    //             //
    //             //     match merged_res {
    //             //         Err(join_err) => {
    //             //             warn!("spawn_blocking error: {:?}", join_err);
    //             //         }
    //             //         Ok(Ok(merged_bytes)) => {
    //             //             // // Send to the MBT wr/**/iter
    //             //             // // We typically store a tile as (Tile, tile_data, Some(content_type)?)
    //             //             // let _ = tx_writer.send((
    //             //             //     parent_tile.into(), // If you have a Tile struct or something
    //             //             //     merged_bytes,
    //             //             //     None
    //             //             // )).await;
    //             //             info!("would send");
    //             //         }
    //             //         Ok(Err(e)) => {
    //             //             warn!("merge_256_tiles error: {:?}", e);
    //             //         }
    //             //     }
    //             // }
    //         })
    //         .await;
    // });
    //
    // // 7) Spawn the writer
    // let writer_handle = tokio::spawn(
    //     writer.write()
    // );
    //
    // // Wait for everything
    // let (proc_res, writer_res) = tokio::join!(proc_handle, writer_handle);
    // proc_res?;
    // writer_res??; // double-? to handle the JoinResult and the actual method Result
    //
    // info!("512ify done in {:?}", start_time.elapsed());

    Ok(())
}
// async fn utiles_512ify2() -> Result<(), Box<dyn std::error::Error>> {
//     println!("utiles ~ 512ify");
//
//     let src_mbtiles = "osm-test-z1.mbtiles";
//     let dst_mbtiles = "osm-test-z1-512.mbtiles";
//
//     let mbt = MbtilesClientAsync::open_existing(src_mbtiles).await?;
//     mbt.assert_mbtiles().await?;
//
//     let (tx, rx) = channel(100);
//
//     mbt.conn(
//         |c| {
//             let mut stmt = c.prepare(QUERY)?;
//             let rows = stmt.query_map([], |row| {
//                 let parent_z: i64 = row.get(0)?;
//                 let parent_x: i64 = row.get(1)?;
//                 let parent_y: i64 = row.get(2)?;
//                 let top_left_data: Vec<u8> = row.get(3)?;
//                 let top_right_data: Vec<u8> = row.get(4)?;
//                 let bottom_left_data: Vec<u8> = row.get(5)?;
//                 let bottom_right_data: Vec<u8> = row.get(6)?;
//
//                 // Send to the channel (blocking_send is fine in this context).
//                 if let Err(e) = tx.blocking_send((
//                     parent_z,
//                     parent_x,
//                     parent_y,
//                     top_left_data,
//                     top_right_data,
//                     bottom_left_data,
//                     bottom_right_data,
//                 )) {
//                     warn!("channel send error: {:?}", e);
//                 }
//
//                 Ok(())
//             })?;
//
//             // consume all rows
//             for row_result in rows {
//                 // You can optionally handle row-level errors here
//                 if let Err(e) = row_result {
//                     error!("row error: {:?}", e);
//                 }
//             }
//                 Ok(())
//         }
//     )
//
//     let (tx_writer, rx_writer) = tokio::sync::mpsc::channel(100);
//     let start_time = std::time::Instant::now();
//     let mut writer = MbtStreamWriterSync {
//         stream: ReceiverStream::new(rx_writer),
//         mbt: Mbtiles::from(dst_mbtiles),
//         stats: MbtWriterStats::default(),
//     };
//
//     Ok(())
// }

#[tokio::main]
async fn main() {
    println!("utiles ~ dev");
    utiles_512ify().await.expect("512ify failed");

    // let r = utiles_dev::quick_maths();
    // if let Err(e) = r {
    //     println!("e: {:?}", e);
    // } else {
    //     println!("2 + 2, that's 4, minus 1 that's 3, quick-maths.");
    // }
}
