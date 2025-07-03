use std::io::Cursor;

use tracing::warn;

use utiles_core::tile_type::{TileFormat, tiletype};

use crate::UtilesResult;

pub mod raster_tile_join;

pub fn load_from_memory(data: &[u8]) -> UtilesResult<image::DynamicImage> {
    image::load_from_memory(data).map_err(|e| e.into())
}

pub fn webpify_image(data: &[u8]) -> UtilesResult<Vec<u8>> {
    match tiletype(data).format {
        TileFormat::Webp => Ok(data.to_vec()),
        TileFormat::Jpg | TileFormat::Png | TileFormat::Gif => {
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

#[must_use]
pub fn image_is_transparent(img: &image::DynamicImage) -> bool {
    match img {
        image::DynamicImage::ImageRgba8(img) => img.pixels().any(|p| p[3] < 255),
        image::DynamicImage::ImageRgba16(img) => img.pixels().any(|p| p[3] < 65535),
        image::DynamicImage::ImageLumaA8(img) => img.pixels().any(|p| p[1] < 255),
        image::DynamicImage::ImageLumaA16(img) => img.pixels().any(|p| p[1] < 65535),
        _ => false,
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
