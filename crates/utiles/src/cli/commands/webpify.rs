use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::ImageEncoder;
use rusqlite::Connection;
use std::io::Cursor;
use tracing::info;

use utiles_core::prelude::*;
use utiles_core::{utile, utile_yup};

use crate::cli::args::WebpifyArgs;
use crate::sqlite::{AsyncSqliteResult, RusqliteResult};
use crate::utilesqlite::mbtiles_async_sqlite::AsyncSqlite;
use crate::utilesqlite::{Mbtiles, MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

fn webpify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
    let img = image::load_from_memory(&data)?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::WebP)?;
    Ok(buf)
}

fn pngify_image(data: &Vec<u8>) -> UtilesResult<Vec<u8>> {
    let img = image::load_from_memory(&data)?;
    let mut buf = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut buf,
        CompressionType::Default,
        FilterType::Adaptive,
    );
    img.write_with_encoder(encoder)?;
    Ok(buf)
}

#[tracing::instrument]
pub async fn webpify_main(args: WebpifyArgs) -> UtilesResult<()> {
    info!("WEBPIFY");
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    let mut dst_mbtiles = Mbtiles::open_new("webp-output.mbtiles", None)?;

    let thingy = mbt
        .conn(move |c: &Connection| -> RusqliteResult<bool> {
            let mut s = c.prepare(
                "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles;",
            )?;

            let mut rows = s.query_map(rusqlite::params![], |row| {
                let z: u8 = row.get(0)?;
                let x: u32 = row.get(1)?;
                let yup: u32 = row.get(2)?;
                let tile = utile_yup!(x, yup, z);
                println!("{tile:?}");
                let tile_data: Vec<u8> = row.get(3)?;
                Ok((tile, tile_data))
                // Ok((z, x, y, tile_data))
            })?;

            for row in rows {
                let (tile, tile_data) = row?;
                println!("{tile:?}");
                // let webp_bytes = webpify_image(&tile_data).map_err(
                //     |e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                // )?;
                let webp_bytes = pngify_image(&tile_data).map_err(|e| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                })?;
                dst_mbtiles.insert_tile_flat::<Tile>(&tile, &webp_bytes)?;
            }
            Ok(true)
        })
        .await?;

    // if args.rm {
    //     mbt.conn(unzxyify).await?;
    // } else {
    //     let zxy_rows_changed = mbt.zxyify().await?;
    //     let json_string = serde_json::to_string_pretty(&zxy_rows_changed)?;
    //     println!("{json_string}");
    // }
    Ok(())
}
