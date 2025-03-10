use crate::{UtilesError, UtilesResult};
use image::{GenericImage, GenericImageView};
use std::io::Cursor;

use super::load_from_memory;

struct RasterTileJoiner {
    pub tl: Option<image::DynamicImage>,
    pub tr: Option<image::DynamicImage>,
    pub bl: Option<image::DynamicImage>,
    pub br: Option<image::DynamicImage>,
}

impl RasterTileJoiner {
    pub(crate) fn preflight(&self) -> UtilesResult<(u32, u32)> {
        if self.tl.is_none()
            && self.tr.is_none()
            && self.bl.is_none()
            && self.br.is_none()
        {
            return Err(UtilesError::AdHoc(
                "one or more images are missing".to_string(),
            ));
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
            Err(UtilesError::AdHoc(
                "images are not all the same size".to_string(),
            ))
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

    pub(crate) fn join_rgb(&self) -> UtilesResult<image::DynamicImage> {
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

    pub(crate) fn join_rgba(&self) -> UtilesResult<image::DynamicImage> {
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

    pub(crate) fn is_transparent(&self) -> bool {
        let non_null_tiles = self.non_null_tiles();
        if non_null_tiles.len() < 4 {
            true
        } else {
            non_null_tiles.len() == 4
                && self
                    .non_null_tiles()
                    .iter()
                    .any(|img| super::image_is_transparent(img))
        }
    }

    pub(crate) fn join(&self) -> UtilesResult<image::DynamicImage> {
        if self.is_transparent() {
            self.join_rgba()
        } else {
            self.join_rgb()
        }
    }
}

pub struct RasterChildren<'a> {
    pub child_0: Option<&'a [u8]>,
    pub child_1: Option<&'a [u8]>,
    pub child_2: Option<&'a [u8]>,
    pub child_3: Option<&'a [u8]>,
}

pub fn dynamic_img_2_webp(img: &image::DynamicImage) -> UtilesResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;
    Ok(bytes)
}

pub fn dynamic_img_2_png(img: &image::DynamicImage) -> UtilesResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    Ok(bytes)
}

pub fn join_raster_children(
    children: &RasterChildren,
) -> UtilesResult<image::DynamicImage> {
    // TIL about `Option::transpose()` which is doppppe
    let top_left = children
        .child_0
        .as_ref()
        .map(|data| load_from_memory(data))
        .transpose()?;
    let top_right = children
        .child_1
        .as_ref()
        .map(|data| load_from_memory(data))
        .transpose()?;
    let bottom_left = children
        .child_2
        .as_ref()
        .map(|data| load_from_memory(data))
        .transpose()?;
    let bottom_right = children
        .child_3
        .as_ref()
        .map(|data| load_from_memory(data))
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
}
