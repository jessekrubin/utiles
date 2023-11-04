use std::collections::HashMap;
use std::hash::Hash;

use rusqlite;
use mbtiles::MbtilesManager;

mod mbtiles;

// impl From<tokio_rusqlite::Error> for Error {
//     fn from(e: tokio_rusqlite::Error) -> Error {
//         Error::RusqliteError(e)
//     }
// }


#[tokio::main]
async fn main() -> tokio_rusqlite::Result<()> {
    // let c_res = Connection::open(
    //     "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles"
    // ).await;

    let filepath= "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles";
    // "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles",
    // "D:\\maps\\reptiles\\mbtiles\\globallandcover.mbtiles",
    // let mbt = MbtilesAsync::open(
    //     "D:\\maps\\reptiles\\mbtiles\\osm\\planet_z0z14_2022_10_13.mbtiles",
    // ).await?;
    //
    // let mdata = mbt.metadata().await?;
    //
    // let mut metadataMap: HashMap<String, Vec<String>> = HashMap::new();
    //
    // for thing in mdata {
    //     println!("{}: {}", thing.name, thing.value);
    //
    //     //     if it does not exist, create empty vector
    //     //     if it does exist, append to vector
    //     let mut v = metadataMap.entry(thing.name).or_insert(Vec::new());
    //     v.push(thing.value);
    // }
    //
    // println!("metadataMap: {:?}", metadataMap);
    //
    // println!("metadata_has_unique_index_name: {}", mbt.metadata_has_unique_index_name().await?);
    //
    let mut mbtiles_manager = MbtilesManager::new();

    // Open the database connection
    mbtiles_manager.open(
        filepath
    ).unwrap();

    let mapfn = |row: &rusqlite::Row| -> rusqlite::Result<String> {
        Ok(row.get(0)?)
    };

    let metadata = mbtiles_manager.metadata();
    // Execute a query
    let result= mbtiles_manager.query("SELECT name, value FROM metadata",
        mapfn
    );
    match result {
        Ok(rows) => {
            for row in rows {
                println!("{}", row);
            }
        }
        Err(err) => eprintln!("Query failed: {}", err),
    }

    println!("metadata: {:?}", metadata);
    // Close the database connection
    mbtiles_manager.close().unwrap();
    //
    // // match c_res {
    // //     Ok(c) => println!("Connection opened"),
    // //     Err(e) => println!("Error opening connection: {}", e),
    // // }
    // let conn = match  c_res {
    //     Ok(c) => c,
    //     Err(e) => return Err(e),
    // };
    //
    // let mdata = conn
    //     .call(|conn| {
    //         let mut stmt = conn.prepare("SELECT name, value FROM metadata")?;
    //         let mdata = stmt
    //             .query_map([], |row| {
    //                 Ok(
    //                     MetadataRow {
    //                         name: row.get(0)?,
    //                         value: row.get(1)?,
    //                     }
    //                 )
    //             })?
    //             .collect::<Result<Vec<MetadataRow>, rusqlite::Error>>()?;
    //
    //         Ok::<_, rusqlite::Error>(mdata)
    //     })
    //     .await?;
    //
    //
    //
    // for thing in mdata {
    //     println!("{}: {}", thing.name, thing.value);
    // }

    Ok(
        ()
    )


    // let mbt = Connection
}
