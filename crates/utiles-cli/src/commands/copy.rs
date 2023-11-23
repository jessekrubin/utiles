use utilesqlite::MbtilesAsync;

pub async fn copy_main() {
    let file = "D:\\maps\\reptiles\\mbtiles\\blue-marble\\blue-marble.mbtiles";
    let mbta = MbtilesAsync::open(file).await.unwrap();
    let r = mbta.metadata_rows().await;
    println!("r: {r:?}");
    let tj = mbta.tilejson().await;
    println!("tj: {tj:?}");
}
