use async_sqlite::{Client, ClientBuilder};
use crate::mbtiles::{mbtiles_metadata};
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;
use utiles::mbtiles::{metadata2tilejson, MinZoomMaxZoom};
use std::error::Error;
use tracing::{error};
use tilejson::TileJSON;

pub struct MbtilesAsync {
    pub client: Client,
}

impl MbtilesAsync {
    pub async fn open(path: &str) -> Result<Self, async_sqlite::Error> {
        let c = ClientBuilder::new().path(path).open().await?;
        Ok(Self { client: c })
    }


    pub async fn metadata_rows(&self) -> Result<Vec<MbtilesMetadataRow>, Box<dyn Error>> {
        let mdrows = self.client.conn(
            |conn| {
                let r = mbtiles_metadata(conn);
                r
            }
        ).await?;
        println!("mdrows: {:?}", mdrows);
        Ok(mdrows)
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

    // pub async fn tilejson(&self) -> Result<TileJSON, Box<dyn Error>>{
    //     // let metadata = self.metadata()?;
    //     let mdrows = self.client.conn(
    // |conn| {
    //
    //     let r = mbtiles_metadata(conn);
    //     r
    //
    // }
    //     ).await?;
    //     println!("mdrows: {:?}", mdrows);
    //     0
    // }
}