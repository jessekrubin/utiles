//! pm(u)tiles
#![cfg(feature = "pmtiles")]

// use std::path::Path;
// use pmtiles::async_reader::AsyncPmTilesReader;
// use tilejson::TileJSON;
//
// use crate::UtilesResult;
//
// pub async fn fspath2pmtilejson<P: AsRef<Path>>(path: P) -> UtilesResult<TileJSON> {
//     let reader = AsyncPmTilesReader::new_with_path(path).await?;
//     let tj = reader.parse_tilejson(vec![]).await?;
//     Ok(tj)
// }
