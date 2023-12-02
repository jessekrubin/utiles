use tracing::warn;

// use rusqlite::Error;
// use utiles::tile_data_row::TileData;
// use utiles::Tile;
// use utilesqlite::MbtilesAsync;
// pub async fn deadpool_testing() -> Result<(), Box<dyn std::error::Error>> {
//     let file = "D:\\maps\\reptiles\\mbtiles\\blue-marble\\blue-marble.mbtiles";
//     let mbta = MbtilesAsync::open(file).await.unwrap();
//     let r = mbta.metadata_rows().await;
//     println!("r: {r:?}");
//     let tj = mbta.tilejson().await;
//     println!("tj: {tj:?}");
//
//     let pool = mbta.pool.get().await.unwrap();
//     let r1 = pool
//         .interact(|conn| {
//             // total tiles
//             let total_tiles: u32 = conn
//                 .query_row("SELECT count(*) FROM tiles", [], |row| row.get(0))
//                 .unwrap();
//             println!("total_tiles: {total_tiles:?}");
//
//             let mut prog = 0;
//             conn.progress_handler(
//                 2000,
//                 Some(move || {
//                     // println!("progress: {:?}", p);
//                     prog += 2000;
//                     println!("prog: {prog:?}");
//                     false
//                 }),
//             );
//             println!("prog: {prog:?}");
//             let mut stmt = conn.prepare(
//                 "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles",
//             )?;
//
//             // .unwrap();
//             let tiles_iter = stmt
//                 .query_map([], |row| {
//                     let z: u8 = row.get(0)?;
//                     let x: u32 = row.get(1)?;
//                     let y: u32 = row.get(2)?;
//                     let data: Vec<u8> = row.get(3)?;
//                     let xyz = Tile::new(x, y, z);
//
//                     let r = TileData { xyz, data };
//
//                     Ok(r)
//                 })?
//                 .collect::<Result<Vec<TileData>, Error>>();
//             tiles_iter
//         })
//         .await;
//     match r1 {
//         Ok(r) => {
//             // println!("r: {:?}", r.len());
//             match r {
//                 Ok(r) => println!("r: {:?}", r.len()),
//                 Err(e) => println!("e: {:?}", e),
//             }
//         }
//         Err(e) => println!("e: {:?}", e),
//     }
//
//     Ok(())
//     // println!("r1: {:?}", r1.len());
// }

/// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗      ██████╗ ███████╗██╗   ██╗
/// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝      ██╔══██╗██╔════╝██║   ██║
/// ██║   ██║   ██║   ██║██║     █████╗  ███████╗█████╗██║  ██║█████╗  ██║   ██║
/// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║╚════╝██║  ██║██╔══╝  ╚██╗ ██╔╝
/// ╚██████╔╝   ██║   ██║███████╗███████╗███████║      ██████╔╝███████╗ ╚████╔╝
///  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝      ╚═════╝ ╚══════╝  ╚═══╝

pub async fn dev_main() -> Result<(), Box<dyn std::error::Error>> {
    warn!("__DEV_MAIN__");
    Ok(())
}
