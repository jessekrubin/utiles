use futures::TryStreamExt;

use geozero::mvt::{Message, Tile};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{query_as, ConnectOptions, Executor, FromRow};

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

fn mvt_dev() {
    let filepath = "D:\\utiles\\crates\\utiles-dev\\12665.vector.pbf";
    //     read to vec of bytes
    let bytes = std::fs::read(filepath).unwrap();

    println!("bytes: {:?}", bytes.len());
    // let buf = bytes.as_slice();

    let cursor = std::io::Cursor::new(bytes.as_slice());

    let mt = Tile::decode(cursor).unwrap();

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

async fn sqlite_deadpool_test() {
    println!("sqlite_deadpool_test");
    let file = "D:\\blue-marble\\blue-marble.mbtiles.NOPE";
    let mbta = utilesqlite::MbtilesAsync::open(file).await.unwrap();

    let tj = mbta.tilejson().await;

    match tj {
        Ok(t) => {
            println!("tj: {t:?}");
        }
        Err(e) => {
            println!("e: {:?}", e);
        }
    }
}

async fn sqlxing() {
    let file = "D:\\blue-marble\\blue-marble.mbtiles";

    let copts = SqliteConnectOptions::new()
        .filename(file)
        .create_if_missing(true);
    let _c = copts.connect().await.unwrap();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(copts)
        .await
        .unwrap();

    // timing
    // start
    let start = std::time::Instant::now();
    let mut r = query_as::<_, MetadataRow>("SELECT * FROM tiles").fetch(&pool);
    while let Some(_row) = r.try_next().await.unwrap() {
        // println!("row: {:?}", row);
    }

    // end
    let end = std::time::Instant::now();
    println!("time: {:?}", end.duration_since(start));

    // as uno fetch
    let start2 = std::time::Instant::now();
    let r2 = query_as::<_, MetadataRow2>("SELECT tile_row FROM tiles")
        .fetch_all(&pool)
        .await
        .unwrap();
    let end2 = std::time::Instant::now();
    println!("r2: {:?}", r2.len());
    println!("time: {:?}", end2.duration_since(start2));
}
#[tokio::main]
async fn main() {
    println!("utiles ~ dev");

    mvt_dev();

    sqlite_deadpool_test().await;

    sqlxing().await;

    // let res = r.iter().map(|row| {
    //     println!("row:j");
    // }).collect::<Vec<()>>();
    // println!("r: {:?}", r);
}
