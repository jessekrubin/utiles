use std::io::Cursor;

use crate::UtilesResult;

pub fn webpify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
    let img = image::load_from_memory(&data)?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::WebP)?;
    Ok(buf)
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
