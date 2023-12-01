use std::error::Error;

use deadpool_sqlite::{self, Config, Pool, PoolError, Runtime};
use tilejson::TileJSON;
use tracing::error;

use utiles::mbtiles::metadata2tilejson;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;

use crate::mbtiles::mbtiles_metadata;

pub struct MbtilesAsync {
    // pub client: Client,
    pub pool: Pool,
}

impl MbtilesAsync {
    pub async fn open(path: &str) -> Result<Self, PoolError> {
        let cfg = Config::new(path);
        let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
        // let conn = pool.get().await.unwrap();
        // let conn = pool.get().await;

        // pool.status()

        // .expect("DB connection failed");
        Ok(Self { pool })
        // let c = ClientBuilder::new().path(path).open().await?;
        // Ok(Self { client: c })
    }

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

    pub async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>> {
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
}