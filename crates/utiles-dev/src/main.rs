use anyhow::Result;
use geozero::{
    mvt::{Message, Tile},
    GeozeroDatasource,
};
mod geojson_builder;
mod mvt2geojson;
mod mvt_commands;
mod mvt_types;
fn mvt_decode(data: &[u8]) -> Result<()> {
    let decoded = Tile::decode(data)?;
    // eprintln!("decoded: {:?}", decoded);

    let utile = utiles::Tile::new(0, 0, 0);

    // convert to geojson
    for layer in decoded.layers {
        // eprintln!("layer: {:?}", layer);

        let extent_f64 = f64::from(layer.extent.unwrap_or(4096));
        // eprintln!("extent_f64: {:?}", extent_f64);

        let mut ut_layer = mvt_types::UtilesMvtLayer {
            inner: &layer,
            xyz: utile,
        };
        let mut buf = Vec::new();
        {
            let mut geojson_writer = geozero::geojson::GeoJsonWriter::new(&mut buf);
            ut_layer.process(&mut geojson_writer)?;
        }

        // eprintln!("geojson");
        let output_str = std::str::from_utf8(&buf).unwrap();
        println!("{}", output_str);
        break;
    }

    // let tile = mvt::Tile::decode(data)?;
    // let tile = tile_types::Tile::from(tile);
    Ok(())
}

fn decode_geojson_builder(data: &[u8]) -> Result<()> {
    let decoded = Tile::decode(data)?;
    // eprintln!("decoded: {:?}", decoded);

    let utile = utiles::Tile::new(0, 0, 0);

    // convert to geojson
    for layer in decoded.layers {
        // eprintln!("layer: {:?}", layer);

        let extent_f64 = f64::from(layer.extent.unwrap_or(4096));
        // eprintln!("extent_f64: {:?}", extent_f64);

        let mut ut_layer = mvt_types::UtilesMvtLayer {
            inner: &layer,
            xyz: utile,
        };
        // let mut buf = Vec::new();
        {
            let mut geojson_builder = geojson_builder::GeoJsonBuilder::new(
                // &mut buf
            );
            ut_layer.process(&mut geojson_builder)?;

            let fc = geojson_builder.collection;
            let stringify = serde_json::to_string(&fc).unwrap();
            println!("{}", stringify);

            // let mut geojson_writer = geozero::geojson::GeoJsonWriter::new(
            // &mut buf
            // );
            // ut_layer.process( &mut geojson_writer)?;
        }

        // eprintln!("geojson");
        // let output_str = std::str::from_utf8(&buf).unwrap();
        // println!(
        // "{}",
        // output_str
        // );
        break;
    }

    // let tile = mvt::Tile::decode(data)?;
    // let tile = tile_types::Tile::from(tile);
    Ok(())
}

fn dev_sync() -> anyhow::Result<()> {
    eprintln!("utiles ~ dev");
    let filepath = "D:\\utiles\\test-data\\tile-types\\0.vector.pbf";

    let read_res = std::fs::read(filepath)?;

    let r = decode_geojson_builder(&read_res);
    // let r = mvt_decode(&read_res);

    Ok(())
}

fn main() {
    let r = dev_sync();
    match r {
        Ok(r) => {
            eprintln!("done ok")
        }
        Err(e) => {
            println!("error");
            // raise the
            println!("e: {:?}", e);
        }
    }
}

// #[tokio::main]
// async fn main() {
//     println!("utiles ~ dev");
//
//     let r = utiles_dev::quick_maths();
//     if let Err(e) = r {
//         println!("e: {:?}", e);
//     } else {
//         println!("2 + 2, that's 4, minus 1 that's 3, quick-maths.");
//     }
// }
