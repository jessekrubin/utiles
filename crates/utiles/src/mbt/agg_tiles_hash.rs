use rusqlite::Connection;

use crate::mbt::hash_types::HashType;
use crate::mbt::TilesFilter;
use crate::UtilesResult;

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
        "SELECT coalesce(
            {hash_type}_concat_hex(
                cast(zoom_level AS text),
                cast(tile_column AS text),
                cast(tile_row AS text),
                tile_data
                ORDER BY zoom_level, tile_column, tile_row),
            {hash_type}_hex(''))
        FROM tiles {where_clause}",
    );
    Ok(sql)
}

pub fn mbt_agg_tiles_hash(
    conn: &Connection,
    hash_type: HashType,
    prefix: Option<&str>,
    filter: &Option<TilesFilter>,
) -> UtilesResult<String> {
    let sql = mbt_agg_tile_hash_query(hash_type, prefix, filter)?;
    let mut stmt = conn.prepare_cached(&sql)?;
    let agg_tiles_hash_str: String = stmt.query_row([], |row| row.get(0))?;
    Ok(agg_tiles_hash_str)
}
//
// pub fn mbt_agg_tiles_hash_md5(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Md5)
// }
//
// pub fn mbt_agg_tiles_hash_fnv1a(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Fnv1a)
// }
//
// pub fn mbt_agg_tiles_hash_xxh32(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Xxh32)
// }
//
// pub fn mbt_agg_tiles_hash_xxh64(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Xxh64)
// }
//
// pub fn mbt_agg_tiles_hash_xxh3_64(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Xxh3_64)
// }
//
// pub fn mbt_agg_tiles_hash_xxh3_128(conn: &Connection) -> RusqliteResult<String> {
//     mbt_agg_tiles_hash(conn, HashType::Xxh3_128)
// }
