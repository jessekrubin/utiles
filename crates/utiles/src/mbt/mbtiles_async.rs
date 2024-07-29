use async_trait::async_trait;
use tilejson::TileJSON;

use utiles_core::{BBox, Tile, TileLike};

use crate::errors::UtilesResult;
use crate::mbt::{MbtMetadataRow, MbtType, MbtilesStats, MetadataChangeFromTo};
use crate::mbt::{MbtilesMetadataJson, MinZoomMaxZoom};
use crate::sqlite::RowsAffected;

#[async_trait]
pub trait MbtilesAsync: Sized {
    fn filepath(&self) -> &str;
    fn filename(&self) -> &str;

    async fn register_utiles_sqlite_functions(&self) -> UtilesResult<()>;
    async fn is_mbtiles_like(&self) -> UtilesResult<bool>;
    async fn is_mbtiles(&self) -> UtilesResult<bool>;
    async fn assert_mbtiles(&self) -> UtilesResult<()>;
    async fn magic_number(&self) -> UtilesResult<u32>;
    async fn tilejson(&self) -> UtilesResult<TileJSON>;
    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtMetadataRow>>;

    async fn metadata_json(&self) -> UtilesResult<MbtilesMetadataJson>;

    /// Returns the metadata row struct for the given name
    async fn metadata_row(&self, name: &str) -> UtilesResult<Option<MbtMetadataRow>>;
    async fn metadata_set(&self, name: &str, value: &str) -> UtilesResult<usize>;
    async fn tiles_is_empty(&self) -> UtilesResult<bool>;

    async fn metadata_minzoom(&self) -> UtilesResult<Option<u8>>;
    async fn metadata_maxzoom(&self) -> UtilesResult<Option<u8>>;

    async fn has_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<bool>;
    async fn query_zxy(&self, z: u8, x: u32, y: u32) -> UtilesResult<Option<Vec<u8>>>;

    async fn query_tile(&self, tile: &Tile) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }

    async fn has_tile(&self, tile: &Tile) -> UtilesResult<bool> {
        self.has_zxy(tile.z(), tile.x(), tile.y()).await
    }

    async fn query_minzoom_maxzoom(&self) -> UtilesResult<Option<MinZoomMaxZoom>>;
    async fn query_tilelike<T: TileLike + Send>(
        &self,
        tile: T,
    ) -> UtilesResult<Option<Vec<u8>>> {
        self.query_zxy(tile.z(), tile.x(), tile.y()).await
    }
    async fn tilejson_ext(&self) -> UtilesResult<TileJSON>;

    async fn query_mbt_type(&self) -> UtilesResult<MbtType>;
    async fn bbox(&self) -> UtilesResult<BBox>;

    async fn zxyify(&self) -> UtilesResult<Vec<RowsAffected>>;

    async fn mbt_stats(&self, full: Option<bool>) -> UtilesResult<MbtilesStats>;

    async fn tiles_count(&self) -> UtilesResult<usize>;
    async fn pragma_encoding(&self) -> UtilesResult<String>;
    async fn metadata_update(
        &self,
        name: &str,
        value: &str,
    ) -> UtilesResult<Option<MetadataChangeFromTo>>;
    async fn update_minzoom_maxzoom(
        &self,
    ) -> UtilesResult<Option<Vec<MetadataChangeFromTo>>>;
}
