use std::error::Error;

use async_trait::async_trait;
use tilejson::TileJSON;

use utiles_core::{Tile, TileLike};
use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;

use crate::errors::UtilesResult;
use crate::UtilesError;

// #[async_trait]
// pub trait Sqlike3Async {
//     // async fn conn(&self) -> &Connection;
//     async fn is_empty_db(&self) -> RusqliteResult<bool>;
//     async fn vacuum(&self) -> RusqliteResult<usize>;
//
//     async fn analyze(&self) -> RusqliteResult<usize>;
//
//     async fn magic_number(&self) -> UtilesResult<u32>;
// }

#[async_trait]
pub trait MbtilesAsync: Sized {
    fn filepath(&self) -> &str;
    fn filename(&self) -> &str;

    async fn open(path: &str) -> UtilesResult<Self>;

    async fn magic_number(&self) -> UtilesResult<u32>;
    async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>>;
    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>>;

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>> {
        let emsg =
            format!("query_zxy not implemented for z: {}, x: {}, y: {}", z, x, y);
        Err(UtilesError::Unimplemented(emsg))
    }

    async fn query_tile(&self, tile: Tile) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }

    async fn query_tilelike<T: TileLike + Send>(
        &self,
        tile: T,
    ) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }
}
