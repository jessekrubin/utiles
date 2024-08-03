use utiles_dev;

// fn mvt_dev() {
//     let filepath = "D:\\utiles\\crates\\utiles-dev\\12665.vector.pbf";
//     //     read to vec of bytes
//     let bytes = std::fs::read(filepath).unwrap();
//
//     println!("bytes: {:?}", bytes.len());
//     // let buf = bytes.as_slice();
//
//     let cursor = std::io::Cursor::new(bytes.as_slice());
//
//     let mt = Tile::decode(cursor).unwrap();
//
//     println!("mt: {:?}", mt);
//     // let t =
//
//     // let gj = mt.to_json().unwrap();
//
//     let num_layers = mt.layers.len();
//     println!("num_layers: {:?}", num_layers);
//
//     // mt.layers
//     // mt.layers.into_iter().map(
//     //     |layer| {
//     //         let mut l = layer.clone();
//     //         println!("l: {:?}", l);
//     //         let s = l.to_json().unwrap();
//     //         println!("s:");
//     //         println!("{}", s);
//     //     }
//     // ).collect::<Vec<()>>();
//
//     // println!("gj: {:?}", gj);
//     // number of layers in tile
//     // let mtjson = serde_json::to_string(&mt).unwrap();
//
//     // let gj = geozero::mvt::to_geojson(&mt).unwrap();
//     // println!("mtjson: {:?}", mtjson);
// }

#[tokio::main]
async fn main() {
    println!("utiles ~ dev");
    let r = utiles_dev::quick_maths();
    if let Err(e) = r {
        println!("e: {:?}", e);
    } else {
        println!("2 + 2, that's 4, minus 1 that's 3, quick-maths.");
    }
}
