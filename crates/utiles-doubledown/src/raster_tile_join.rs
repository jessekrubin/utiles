use image::{GenericImage, GenericImageView};
use std::io::Cursor;

fn load_image_from_memory(data: &[u8]) -> anyhow::Result<image::DynamicImage> {
    image::load_from_memory(data)
        .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))
}

pub fn image_is_transparent(img: &image::DynamicImage) -> bool {
    match img {
        image::DynamicImage::ImageRgba8(img) => img.pixels().any(|p| p[3] < 255),
        image::DynamicImage::ImageRgba16(img) => img.pixels().any(|p| p[3] < 255),
        image::DynamicImage::ImageLumaA8(img) => img.pixels().any(|p| p[1] < 255),
        image::DynamicImage::ImageLumaA16(img) => img.pixels().any(|p| p[1] < 255),
        _ => false,
    }
}

// pub fn image_unique_pixel_count(img: &image::DynamicImage) -> usize {
//     match img {
//         image::DynamicImage::ImageRgba8(img) => img.pixels().collect::<std::collections::HashSet<_>>().len(),
//         image::DynamicImage::ImageRgba16(img) => img.pixels().collect::<std::collections::HashSet<_>>().len(),
//         image::DynamicImage::ImageLumaA8(img) => img.pixels().collect::<std::collections::HashSet<_>>().len(),
//         image::DynamicImage::ImageLumaA16(img) => img.pixels().collect::<std::collections::HashSet<_>>().len(),
//         _ => 0,
//     }
// }
//
// pub fn image_could_be_paletted_png(img: &image::DynamicImage) -> bool {
//     match img {
//         image::DynamicImage::ImageRgba8(img) => image_unique_pixel_count(&image::DynamicImage::ImageRgba8(img.clone())) <= 256,
//         image::DynamicImage::ImageRgba16(img) => image_unique_pixel_count(&image::DynamicImage::ImageRgba16(img.clone())) <= 256,
//         image::DynamicImage::ImageLumaA8(img) => image_unique_pixel_count(&image::DynamicImage::ImageLumaA8(img.clone())) <= 256,
//         image::DynamicImage::ImageLumaA16(img) => image_unique_pixel_count(&image::DynamicImage::ImageLumaA16(img.clone())) <= 256,
//         _ => false,
//     }
// }
struct RasterTileJoiner {
    pub tl: Option<image::DynamicImage>,
    pub tr: Option<image::DynamicImage>,
    pub bl: Option<image::DynamicImage>,
    pub br: Option<image::DynamicImage>,
}
impl RasterTileJoiner {
    pub fn preflight(
        &self,
    ) -> anyhow::Result<
        //     dims
        (u32, u32),
    > {
        if self.tl.is_none()
            && self.tr.is_none()
            && self.bl.is_none()
            && self.br.is_none()
        {
            return Err(anyhow::anyhow!("one or more images are missing"));
        }

        // if all images are the same size, return the size... otherwise no go err
        let sizes: Vec<(u32, u32)> = self
            .non_null_tiles()
            .iter()
            .map(|img| img.dimensions())
            .collect();

        if sizes.iter().all(|&x| x == sizes[0]) {
            Ok(sizes[0])
        } else {
            Err(anyhow::anyhow!("images are not all the same size"))
        }
    }

    fn tiles_vec(&self) -> Vec<&Option<image::DynamicImage>> {
        vec![&self.tl, &self.tr, &self.bl, &self.br]
    }

    fn non_null_tiles(&self) -> Vec<&image::DynamicImage> {
        self.tiles_vec()
            .into_iter() // into iter
            .filter_map(|x| x.as_ref()) // filter out the Nones
            .collect() // collect into a vec
    }

    pub fn join_rgb(&self) -> anyhow::Result<image::DynamicImage> {
        let (w, h) = self.preflight()?;

        let out_w = w * 2;
        let out_h = h * 2;
        let mut img_buf_b = image::DynamicImage::new_rgb8(out_w, out_h);
        // if tl is not none, copy it to the top left
        if let Some(tl) = &self.tl {
            img_buf_b.copy_from(tl, 0, 0)?;
        }

        // if tr is not none, copy it to the top right
        if let Some(tr) = &self.tr {
            img_buf_b.copy_from(tr, w, 0)?;
        }
        // if bl is not none, copy it to the bottom left
        if let Some(bl) = &self.bl {
            img_buf_b.copy_from(bl, 0, h)?;
        }
        // if br is not none, copy it to the bottom right
        if let Some(br) = &self.br {
            img_buf_b.copy_from(br, h, w)?;
        }
        Ok(img_buf_b)
    }

    pub fn join_rgba(&self) -> anyhow::Result<image::DynamicImage> {
        let (w, h) = self.preflight()?;

        let out_w = w * 2;
        let out_h = h * 2;
        let mut img_buf_b = image::DynamicImage::new_rgba8(out_w, out_h);
        // if tl is not none, copy it to the top left
        if let Some(tl) = &self.tl {
            img_buf_b.copy_from(tl, 0, 0)?;
        }

        // if tr is not none, copy it to the top right
        if let Some(tr) = &self.tr {
            img_buf_b.copy_from(tr, w, 0)?;
        }
        // if bl is not none, copy it to the bottom left
        if let Some(bl) = &self.bl {
            img_buf_b.copy_from(bl, 0, h)?;
        }
        // if br is not none, copy it to the bottom right
        if let Some(br) = &self.br {
            img_buf_b.copy_from(br, h, w)?;
        }
        Ok(img_buf_b)
    }

    pub fn is_transparent(&self) -> bool {
        let non_null_tiles = self.non_null_tiles();
        non_null_tiles.len() == 4
            && self
                .non_null_tiles()
                .iter()
                .any(|img| image_is_transparent(img))
    }

    pub fn join(&self) -> anyhow::Result<image::DynamicImage> {
        if self.is_transparent() {
            self.join_rgba()
        } else {
            self.join_rgb()
        }
    }
}

#[expect(clippy::struct_field_names)]
pub struct RasterChildren<'a> {
    pub child_0: Option<&'a [u8]>,
    pub child_1: Option<&'a [u8]>,
    pub child_2: Option<&'a [u8]>,
    pub child_3: Option<&'a [u8]>,
}

pub fn dynamic_img_2_webp(img: &image::DynamicImage) -> anyhow::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    Ok(bytes)
}

// pub fn dynamic_img_2_png(img: &DynamicImage) -> anyhow::Result<Vec<u8>> {
//     let mut bytes: Vec<u8> = Vec::new();
//     img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
//     Ok(bytes)
// }

pub fn join_raster_children(
    children: &RasterChildren,
) -> anyhow::Result<image::DynamicImage> {
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
    let joiner = RasterTileJoiner {
        tl: top_left,
        tr: top_right,
        bl: bottom_left,
        br: bottom_right,
    };
    let img_buf = joiner.join()?;
    Ok(img_buf)
    // Buffer the result in memory
    // let mut bytes: Vec<u8> = Vec::new();
    // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    // Ok(bytes)
}
//////////////////////////////////////////////////////////////////////////////
// experiment with `image-merger` crate
// NOTE: does not actually seem to be faster at all
//////////////////////////////////////////////////////////////////////////////
// fn generate_known_image(
//     height: u32,
//     width: u32,
// ) -> image_merger::BufferedImage<image::Rgba<u8>> {
//     // Create an image buffer with the given dimensions
//     image_merger::BufferedImage::new_from_pixel(
//         width,
//         height,
//         image::Rgba([255, 0, 0, 255]),
//     )
// }
// pub fn join_raster_children_external_dep(
//     children: &RasterChildren,
// ) -> anyhow::Result<Vec<u8>> {
//     use image_merger::Merger;
//     // pub fn join_raster_children<T>(children: &T) -> anyhow::Result<Vec<u8>>
//     // where
//     //     T: for<'a> ChildTiles<TileData = Option<&'a [u8]>>,
//     // {
//     // pub fn join_images(
//     //     children: &TileChildrenRow<ChildTilesData>,
//     // ) -> anyhow::Result<(Tile, Vec<u8>)> {
//     // Helper function to load an image from memory with error handling
//     // TIL about `Option::transpose()` which is doppppe
//     let top_left = children
//         .child_0
//         .as_ref()
//         .map(|data| load_image_from_memory(data))
//         .transpose()?;
//     let top_right = children
//         .child_1
//         .as_ref()
//         .map(|data| load_image_from_memory(data))
//         .transpose()?;
//     let bottom_left = children
//         .child_2
//         .as_ref()
//         .map(|data| load_image_from_memory(data))
//         .transpose()?;
//     let bottom_right = children
//         .child_3
//         .as_ref()
//         .map(|data| load_image_from_memory(data))
//         .transpose()?;
//     let mut merger: image_merger::KnownSizeMerger<image::Rgba<u8>, _> =
//         image_merger::KnownSizeMerger::new((512, 512), 2, 4, None);
//
//     let default_image = generate_known_image(256, 256);
//     let top_left_buffered: Option<image_merger::BufferedImage<image::Rgba<u8>>> =
//         top_left.map(|img| image_merger::BufferedImage::from(img.to_rgba8()));
//     let top_right_buffered: Option<image_merger::BufferedImage<image::Rgba<u8>>> =
//         top_right.map(|img| image_merger::BufferedImage::from(img.to_rgba8()));
//     let bottom_left_buffered: Option<image_merger::BufferedImage<image::Rgba<u8>>> =
//         bottom_left.map(|img| image_merger::BufferedImage::from(img.to_rgba8()));
//     let bottom_right_buffered: Option<image_merger::BufferedImage<image::Rgba<u8>>> =
//         bottom_right.map(|img| image_merger::BufferedImage::from(img.to_rgba8()));
//
//     let top_left_buffered_ref: &image_merger::BufferedImage<image::Rgba<u8>> =
//         top_left_buffered.as_ref().unwrap_or(&default_image);
//     let top_right_buffered_ref: &image_merger::BufferedImage<image::Rgba<u8>> =
//         top_right_buffered.as_ref().unwrap_or(&default_image);
//     let bottom_left_buffered_ref: &image_merger::BufferedImage<image::Rgba<u8>> =
//         bottom_left_buffered.as_ref().unwrap_or(&default_image);
//     let bottom_right_buffered_ref: &image_merger::BufferedImage<image::Rgba<u8>> =
//         bottom_right_buffered.as_ref().unwrap_or(&default_image);
//
//     // vec of images or the default image
//     let images: Vec<&image_merger::BufferedImage<image::Rgba<u8>>> = vec![
//         top_left_buffered_ref,
//         top_right_buffered_ref,
//         bottom_left_buffered_ref,
//         bottom_right_buffered_ref,
//     ];
//     merger.bulk_push(&images);
//     let canvas = merger.get_canvas();
//
//     // Join the images
//     // let joiner = ImgJoiner {
//     //     tl: top_left,
//     //     tr: top_right,
//     //     bl: bottom_left,
//     //     br: bottom_right,
//     // };
//     // let img_buf = joiner.join()?;
//     // Buffer the result in memory
//     let mut bytes: Vec<u8> = Vec::new();
//     canvas.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
//     // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
//     // img_buf.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
//     Ok(bytes)
// }
