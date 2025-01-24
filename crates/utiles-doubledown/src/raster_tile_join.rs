use crate::ImgJoiner;
use std::io::Cursor;
fn load_image_from_memory(data: &[u8]) -> anyhow::Result<image::DynamicImage> {
    image::load_from_memory(data)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))
}
// pub trait ChildTiles {
//     type TileData;
//
//     fn child_0(&self) -> Self::TileData;
//     fn child_1(&self) -> Self::TileData;
//     fn child_2(&self) -> Self::TileData;
//     fn child_3(&self) -> Self::TileData;
// }

#[allow(clippy::struct_field_names)]
pub struct RasterChildren<'a> {
    pub child_0: Option<&'a [u8]>,
    pub child_1: Option<&'a [u8]>,
    pub child_2: Option<&'a [u8]>,
    pub child_3: Option<&'a [u8]>,
}
pub fn join_raster_children(children: &RasterChildren) -> anyhow::Result<Vec<u8>> {
    // pub fn join_raster_children<T>(children: &T) -> anyhow::Result<Vec<u8>>
    // where
    //     T: for<'a> ChildTiles<TileData = Option<&'a [u8]>>,
    // {
    // pub fn join_images(
    //     children: &TileChildrenRow<ChildTilesData>,
    // ) -> anyhow::Result<(Tile, Vec<u8>)> {
    // Helper function to load an image from memory with error handling
    // TIL about `Option::transpose()` which is doppppe
    let top_left = children
        .child_0
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let top_right = children
        .child_1
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let bottom_left = children
        .child_2
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;
    let bottom_right = children
        .child_3
        .as_ref()
        .map(|data| load_image_from_memory(data))
        .transpose()?;

    // Join the images
    let joiner = ImgJoiner {
        tl: top_left,
        tr: top_right,
        bl: bottom_left,
        br: bottom_right,
    };

    let img_buf = joiner.join()?;

    // Buffer the result in memory
    let mut bytes: Vec<u8> = Vec::new();
    // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    Ok(bytes)
}
