use crate::errors::UtilesResult;
use crate::mbt::hash_types::HashType;
use crate::mbt::{make_tiles_stream, TilesFilter};
use crate::utilesqlite::MbtilesAsyncSqliteClient;
use futures::StreamExt;
use hex::ToHex;
use md5::Digest;
use noncrypto_digests::Fnv;
use rusqlite::Connection;
use serde::Serialize;
use tokio_stream::wrappers::ReceiverStream;
use tracing::debug;
use utiles_core::{Tile, TileLike};

#[derive(Debug, Serialize)]
pub struct AggHashResult {
    pub hash_type: HashType,
    pub hash: String,
    pub ntiles: usize,
    pub dt: std::time::Duration,
}
// =================================================================
// HASH FUNCTIONS ~ HASH FUNCTIONS ~ HASH FUNCTIONS ~ HASH FUNCTIONS
// =================================================================
// pub fn mbt_agg_tile_hash_query(hash_type: HashType) -> String {
//     let sql = format!(
//         "SELECT coalesce(
//             {hash_type}_concat_hex(
//                 cast(zoom_level AS text),
//                 cast(tile_column AS text),
//                 cast(tile_row AS text),
//                 tile_data
//                 ORDER BY zoom_level, tile_column, tile_row),
//             {hash_type}_hex(''))
//         FROM tiles"
//     );
//     sql
// }
pub fn mbt_agg_tile_hash_query(
    hash_type: HashType,
    prefix: Option<&str>,
    filter: &Option<TilesFilter>,
) -> UtilesResult<String> {
    let where_clause = if let Some(filter) = filter {
        filter.mbtiles_sql_where(prefix)?
    } else {
        String::new()
    };
    let sql = format!(
        "
SELECT
    coalesce(
        {hash_type}_concat_hex(
            cast(zoom_level AS text),
            cast(tile_column AS text),
            cast(tile_row AS text),
            tile_data
            ORDER BY zoom_level, tile_column, tile_row
        ),
        {hash_type}_hex('')
    ) AS concatenated_hash,
    COUNT(*) AS total_count
FROM tiles
{where_clause}
    ",
    );
    Ok(sql)
}

pub fn mbt_agg_tiles_hash_query(
    conn: &Connection,
    hash_type: HashType,
    prefix: Option<&str>,
    filter: &Option<TilesFilter>,
) -> UtilesResult<AggHashResult> {
    let sql = mbt_agg_tile_hash_query(hash_type, prefix, filter)?;
    debug!("sql: {:?}", sql);
    let mut stmt = conn.prepare_cached(&sql)?;
    // start time
    let ti = std::time::Instant::now();
    let (agg_tiles_hash_str, count): (String, i64) =
        stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let dt = ti.elapsed();
    Ok(AggHashResult {
        hash_type,
        hash: agg_tiles_hash_str,
        ntiles: count as usize,
        dt,
    })
}

pub async fn hash_stream<T: Digest>(
    mut data: ReceiverStream<Vec<u8>>,
) -> (String, usize) {
    let mut hasher = T::new();
    let mut count = 0;
    while let Some(chunk) = data.next().await {
        hasher.update(&chunk);
        count += 1;
    }
    // hasher.update(data);
    (hasher.finalize().to_vec().encode_hex_upper(), count)
}
pub async fn tile_stream_to_bytes_stream(
    mut data: ReceiverStream<(Tile, Vec<u8>)>,
) -> ReceiverStream<Vec<u8>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        while let Some((tile, tile_data)) = data.next().await {
            let bytes = vec![
                tile.z().to_string().as_bytes().to_vec(),
                tile.x().to_string().as_bytes().to_vec(),
                tile.yup().to_string().as_bytes().to_vec(),
                tile_data,
            ]
            .concat();
            tx.send(bytes).await.unwrap();
        }
    });
    ReceiverStream::new(rx)
}
impl HashType {
    async fn hash_stream(&self, data: ReceiverStream<Vec<u8>>) -> (String, usize) {
        match self {
            HashType::Md5 => hash_stream::<md5::Md5>(data).await,
            HashType::Fnv1a => hash_stream::<Fnv>(data).await,
            HashType::Xxh32 => hash_stream::<noncrypto_digests::Xxh32>(data).await,
            HashType::Xxh64 => hash_stream::<noncrypto_digests::Xxh64>(data).await,
            HashType::Xxh3_64 => hash_stream::<noncrypto_digests::Xxh3_64>(data).await,
            HashType::Xxh3_128 => {
                hash_stream::<noncrypto_digests::Xxh3_128>(data).await
            }
        }
    }
}

pub async fn mbt_agg_tiles_hash_stream(
    mbt: &MbtilesAsyncSqliteClient,
    hash_type: HashType,
    prefix: Option<&str>,
    filter: &Option<TilesFilter>,
) -> UtilesResult<AggHashResult> {
    // let sql = mbt_agg_tile_hash_query(hash_type, prefix, filter)?;
    // debug!("sql: {:?}", sql);
    // let mut stmt = conn.prepare_cached(&sql)?;
    // start time

    let where_clause = if let Some(filter) = filter {
        filter.mbtiles_sql_where(prefix)?
    } else {
        String::new()
    };
    let query = format!(
        "SELECT zoom_level, tile_column, tile_row, tile_data FROM tiles {} ORDER BY zoom_level, tile_column, tile_row;",
        where_clause
    );
    let ti = std::time::Instant::now();
    let stream = make_tiles_stream(&mbt, Some(query.as_str()))?;
    let bstream = tile_stream_to_bytes_stream(stream).await;

    let (agg_tiles_hash_str, ntiles) = hash_type.hash_stream(bstream).await;

    // let (agg_tiles_hash_str, count): (String, i64) =
    //     stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let dt = ti.elapsed();
    Ok(AggHashResult {
        hash_type,
        hash: agg_tiles_hash_str,
        ntiles,
        dt,
    })
}
