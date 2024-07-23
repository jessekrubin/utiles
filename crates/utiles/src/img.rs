use crate::{UtilesError, UtilesResult};
use std::io::Cursor;
use tracing::warn;
use utiles_core::tile_type::{tiletype, TileType};

pub fn webpify_image(data: &[u8]) -> UtilesResult<Vec<u8>> {
    match tiletype(data) {
        TileType::Webp => Ok(data.to_vec()),
        TileType::Jpg | TileType::Png | TileType::Gif => {
            let img = image::load_from_memory(data)?;

            let mut buf = Vec::new();

            img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::WebP)?;
            Ok(buf)
        }
        _ => {
            warn!("Unsupported image type");
            Ok(data.to_vec())
        }
    }
}

pub fn oxipngify(data: &[u8], options: &oxipng::Options) -> UtilesResult<Vec<u8>> {
    if let TileType::Png = tiletype(data) {
        oxipng::optimize_from_memory(data, options).map_err(UtilesError::OxipngError)
    } else {
        warn!("Unsupported image type");
        Ok(data.to_vec())
    }
}

// TODO: Implement pngify_image
// fn pngify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
//     let img = image::load_from_memory(&data)?;
//     let mut buf = Vec::new();
//     let encoder = PngEncoder::new_with_quality(
//         &mut buf,
//         CompressionType::Default,
//         FilterType::Adaptive,
//     );
//     img.write_with_encoder(encoder)?;
//     Ok(buf)
// }
