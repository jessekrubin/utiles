use crate::mbt::hash_types::HashType;
use crate::mbt::TilesFilter;
use crate::UtilesResult;
use rusqlite::Connection;
use serde::Serialize;
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
LIMIT 1
    ",
    );
    Ok(sql)
}

pub fn mbt_agg_tiles_hash(
    conn: &Connection,
    hash_type: HashType,
    prefix: Option<&str>,
    filter: &Option<TilesFilter>,
) -> UtilesResult<AggHashResult> {
    let sql = mbt_agg_tile_hash_query(hash_type, prefix, filter)?;
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
