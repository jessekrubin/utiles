use std::error::Error;

use async_trait::async_trait;
use deadpool_sqlite::{self, Config, Pool, Runtime};
use tilejson::TileJSON;
use tracing::error;

use utiles_core::mbutiles::metadata_row::MbtilesMetadataRow;
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
    async fn open(path: &str) -> UtilesResult<Self> {
        let cfg = Config::new(path);
        let pool = cfg.create_pool(Runtime::Tokio1);

        // let conn = pool.get().await.unwrap();
        // let conn = pool.get().await;

        // pool.status()

        // .expect("DB connection failed");
        match pool {
            Ok(p) => Ok(Self {
                pool: p,
                dbpath: DbPath::new(path),
            }),
            Err(e) => Err(UtilesError::Unknown(e.to_string())),
        }
    }
    fn filepath(&self) -> &str {
        &self.dbpath.fspath
    }
    fn filename(&self) -> &str {
        &self.dbpath.filename
    }
    async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
        let metadata = self.metadata_rows().await?;
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

    // async fn select_tile<T: TileLike>(&self, tile: T) -> UtilesResult<Vec<u8>> {
    //     Err(UtilesError::Unimplemented("deadpool select_tile".to_string()))
    // }
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
