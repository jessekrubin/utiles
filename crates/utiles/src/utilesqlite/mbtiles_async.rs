use async_trait::async_trait;
use tilejson::TileJSON;

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;
use utiles_core::mbutiles::MinZoomMaxZoom;
use utiles_core::{Tile, TileLike};

use crate::errors::UtilesResult;

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

    async fn register_utiles_sqlite_functions(&self) -> UtilesResult<()>;
    async fn is_mbtiles(&self) -> UtilesResult<bool>;
    async fn magic_number(&self) -> UtilesResult<u32>;
    async fn tilejson(&self) -> UtilesResult<TileJSON>;
    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>>;
    async fn metadata_row(
        &self,
        name: &str,
    ) -> UtilesResult<Option<MbtilesMetadataRow>>;
    async fn metadata_set(&self, name: &str, value: &str) -> UtilesResult<usize>;
    async fn tiles_is_empty(&self) -> UtilesResult<bool>;

    async fn metadata_minzoom(&self) -> UtilesResult<Option<u8>>;
    async fn metadata_maxzoom(&self) -> UtilesResult<Option<u8>>;

    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>>;

    async fn query_tile(&self, tile: Tile) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }

    async fn query_minzoom_maxzoom(&self) -> UtilesResult<Option<MinZoomMaxZoom>>;
    async fn query_tilelike<T: TileLike + Send>(
        &self,
        tile: T,
    ) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }
    async fn tilejson_ext(&self) -> UtilesResult<TileJSON>;
}
