use std::error::Error;

use async_trait::async_trait;
use deadpool_sqlite::{self, Pool};
use tilejson::TileJSON;
use tracing::error;

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;
use utiles_core::mbutiles::MinZoomMaxZoom;
use utiles_core::tile_data_row::TileData;
use utiles_core::Tile;

use crate::errors::UtilesResult;
use crate::utilejson::metadata2tilejson;
use crate::utilesqlite::dbpath::DbPath;
use crate::utilesqlite::mbtiles::{
    insert_tile_flat_mbtiles, insert_tiles_flat_mbtiles, mbtiles_metadata,
};
use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::UtilesError;

pub struct MbtilesDeadpool {
    // pub client: Client,
    pub pool: Pool,
    pub dbpath: DbPath,
}

#[async_trait]
impl MbtilesAsync for MbtilesDeadpool {
    fn filepath(&self) -> &str {
        &self.dbpath.fspath
    }
    fn filename(&self) -> &str {
        &self.dbpath.filename
    }
    async fn tilejson(&self) -> UtilesResult<TileJSON> {
        let metadata = self.metadata_rows().await.map_err(|e| {
            error!("Error getting metadata rows: {}", e);
            UtilesError::Unknown(e.to_string())
        })?;
        let tj = metadata2tilejson(metadata);
        match tj {
            Ok(t) => Ok(t),
            Err(e) => {
                error!("Error parsing metadata to TileJSON: {}", e);
                Err(e)
            }
        }
    }

    async fn metadata_rows(&self) -> UtilesResult<Vec<MbtilesMetadataRow>> {
        let c = self.pool.get().await.unwrap();
        let r = c
            .interact(|conn| mbtiles_metadata(conn))
            .await
            .map_err(|e| UtilesError::Unknown(e.to_string()))?;
        match r {
            Ok(mdrows) => Ok(mdrows),
            Err(e) => Err(e.into()),
        }
    }

    async fn magic_number(&self) -> UtilesResult<u32> {
        todo!()
    }

    async fn query_zxy(
        &self,
        _z: u8,
        _x: u32,
        _y: u32,
    ) -> UtilesResult<Option<Vec<u8>>> {
        todo!()
    }

    async fn metadata_row(
        &self,
        _name: &str,
    ) -> UtilesResult<Option<MbtilesMetadataRow>> {
        todo!()
    }

    async fn query_minzoom_maxzoom(&self) -> UtilesResult<Option<MinZoomMaxZoom>> {
        todo!()
    }

    async fn tilejson_ext(&self) -> UtilesResult<TileJSON> {
        todo!()
    }

    async fn is_mbtiles(&self) -> UtilesResult<bool> {
        todo!()
    }

    async fn register_utiles_sqlite_functions(&self) -> UtilesResult<()> {
        todo!()
    }

    async fn metadata_set(&self, _name: &str, _value: &str) -> UtilesResult<usize> {
        todo!()
    }

    async fn tiles_is_empty(&self) -> UtilesResult<bool> {
        todo!()
    }

    async fn metadata_minzoom(&self) -> UtilesResult<Option<u8>> {
        todo!()
    }

    async fn metadata_maxzoom(&self) -> UtilesResult<Option<u8>> {
        todo!()
    }
}

impl MbtilesDeadpool {
    pub async fn metadata_rows(
        &self,
    ) -> Result<Vec<MbtilesMetadataRow>, Box<dyn Error>> {
        let c = self.pool.get().await.unwrap();
        let r = c
            .interact(|conn| {
                mbtiles_metadata(conn)
                // let mdrows = mbtiles_metadata(conn);
                // mdrows
            })
            .await;

        // let mdrows = self.client.conn(|conn| mbtiles_metadata(conn)).await?;
        // println!("mdrows: {:?}", mdrows);
        // Ok(vec![])
        Ok(r??)
    }

    // pub async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
    //     let metadata = self.metadata_rows().await?;
    //     let tj = metadata2tilejson(metadata);
    //     match tj {
    //         Ok(t) => Ok(t),
    //         Err(e) => {
    //             error!("Error parsing metadata to TileJSON: {}", e);
    //             Err(e)
    //         }
    //     }
    // }

    pub async fn insert_tile(
        &self,
        tile: Tile,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let c = self.pool.get().await.unwrap();
        let _interaction_res = c
            .interact(move |conn| {
                // Assuming insert_tile_flat_mbtiles is a synchronous function
                insert_tile_flat_mbtiles(conn, tile, data).map_err(|e| {
                    error!("Error inserting tile: {}", e);
                    e
                })
            })
            .await?;

        Ok(())
    }

    pub async fn insert_tiles_flat(
        &self,
        tiles: Vec<TileData>,
    ) -> Result<(), Box<dyn Error>> {
        let c = self.pool.get().await.unwrap();
        let interaction_res = c
            .interact(move |conn| {
                // Assuming insert_tile_flat_mbtiles is a synchronous function

                insert_tiles_flat_mbtiles(conn, &tiles, None)
            })
            .await?;
        println!("interaction_res: {:?}", interaction_res);
        Ok(())
    }
}
