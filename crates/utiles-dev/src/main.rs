use futures::TryStreamExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{query, query_as, query_as_unchecked, ConnectOptions, FromRow, Statement};
use std::fmt::Pointer;
use geozero;
use geozero::mvt::{self, Message, Tile, tile};
use geozero::{ToJson,ProcessToJson};

// #[derive(Debug, FromRow)]
// struct MetadataRow {
//     name: String,
//     value: String,
// }
#[derive(Debug, FromRow)]
struct MetadataRow2 {
    tile_row: i32,
    // tile_column: i32,
    // zoom_level: i32,
    // tile_data: Vec<u8>,
}
#[derive(Debug, FromRow)]
struct MetadataRow {
    tile_row: i32,
    tile_column: i32,
    zoom_level: i32,
    tile_data: Vec<u8>,
}

fn mvt_dev(){
    let filepath = "D:\\utiles\\crates\\utiles-dev\\12665.vector.pbf";
    //     read to vec of bytes
    let bytes: &[u8] = std::fs::read(filepath).unwrap().as_slice();
    println!("bytes: {:?}", bytes.len());
    let buf = bytes.as_slice();

    // let mut cursor = std::io::Cursor::new(buf);

    let mt = Tile::decode(&bytes).unwrap();

    println!("mt: {:?}", mt);
    // let t =

    // let gj = mt.to_json().unwrap();

    let num_layers = mt.layers.len();
    println!("num_layers: {:?}", num_layers);

    // mt.layers
    // mt.layers.into_iter().map(
    //     |layer| {
    //         let mut l = layer.clone();
    //         println!("l: {:?}", l);
    //         let s = l.to_json().unwrap();
    //         println!("s:");
    //         println!("{}", s);
    //     }
    // ).collect::<Vec<()>>();


    // println!("gj: {:?}", gj);
    // number of layers in tile
    // let mtjson = serde_json::to_string(&mt).unwrap();

    // let gj = geozero::mvt::to_geojson(&mt).unwrap();
    // println!("mtjson: {:?}", mtjson);
}

#[tokio::main]
async fn main() {
    println!("utiles ~ dev");


    // let file = "D:\\maps\\reptiles\\mbtiles\\blue-marble\\blue-marble.mbtiles";
    //
    // let copts = SqliteConnectOptions::new()
    //     .filename(file)
    //     .create_if_missing(true);
    // let mut c = copts.connect().await.unwrap();
    //
    // let pool = SqlitePoolOptions::new()
    //     .max_connections(5)
    //     .connect_with(copts)
    //     .await
    //     .unwrap();
    //
    // // timing
    // // start
    // let start = std::time::Instant::now();
    // let mut r = query_as::<_, MetadataRow>("SELECT * FROM tiles").fetch(&pool);
    // while let Some(row) = r.try_next().await.unwrap() {
    //     // println!("row: {:?}", row);
    // }
    //
    // // end
    // let end = std::time::Instant::now();
    // println!("time: {:?}", end.duration_since(start));
    //
    // // as uno fetch
    // let start2 = std::time::Instant::now();
    // let r2 = query_as::<_, MetadataRow2>("SELECT tile_row FROM tiles")
    //     .fetch_all(&pool)
    //     .await
    //     .unwrap();
    // let end2 = std::time::Instant::now();
    // println!("r2: {:?}", r2.len());
    // println!("time: {:?}", end2.duration_since(start2));

    mvt_dev();

    // let res = r.iter().map(|row| {
    //     println!("row:j");
    // }).collect::<Vec<()>>();
    // println!("r: {:?}", r);
}
