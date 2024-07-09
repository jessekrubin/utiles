use crate::sqlite::{AffectedType, RowsAffected, RusqliteResult};
use rusqlite::Connection;
use tracing::debug;

const HAS_ZY_MAP_QUERY: &str = include_str!("sql/has-zy-map-query.sql");

/// return true if the mbtiles has a table called `_zy_map`
///
/// `_zy_map` columns are `z`, `y`, `yup` all integers
pub fn has_zy_map(conn: &Connection) -> RusqliteResult<bool> {
    let mut stmt = conn.prepare(HAS_ZY_MAP_QUERY)?;
    let r = stmt.query_row([], |row| {
        let a: i64 = row.get(0)?;
        Ok(a)
    })?;
    Ok(r == 1)
}

pub fn create_zy_map_table(conn: &Connection) -> RusqliteResult<()> {
    conn.execute(
        "CREATE TABLE _zy_map (z INTEGER NOT NULL, y INTEGER NOT NULL, yup INTEGER NOT NULL, PRIMARY KEY (z, y))",
        [],
    )?;
    Ok(())
}

pub fn create_zy_map_index(conn: &Connection) -> RusqliteResult<()> {
    conn.execute("CREATE UNIQUE INDEX _zy_map_ix ON _zy_map (z, y)", [])?;
    Ok(())
}

pub fn drop_zy_map_table(conn: &Connection) -> RusqliteResult<()> {
    conn.execute("DROP TABLE _zy_map", [])?;
    Ok(())
}

pub fn drop_zy_map_index(conn: &Connection) -> RusqliteResult<()> {
    conn.execute("DROP INDEX _zy_map_ix", [])?;
    Ok(())
}

pub fn update_zy_map(conn: &Connection) -> RusqliteResult<usize> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO _zy_map (z, y, yup) SELECT distinct zoom_level as z, tile_row as y, (1 << zoom_level - 1 - tile_row) as yup FROM tiles;",
        [],
    )?;
    Ok(n)
}

pub fn create_zxy_view(conn: &Connection) -> RusqliteResult<()> {
    conn.execute(
        "
CREATE VIEW zxy AS
SELECT
  _zy_map.z AS z,
  tiles.tile_column AS x,
  _zy_map.yup AS y,
  tiles.tile_data AS tile_data
FROM
  _zy_map
JOIN
  tiles ON _zy_map.z = tiles.zoom_level
        AND _zy_map.y = tiles.tile_row;
",
        [],
    )?;
    Ok(())
}

pub fn drop_zxy_view(conn: &Connection) -> RusqliteResult<()> {
    conn.execute("DROP VIEW zxy", [])?;
    Ok(())
}

pub fn deorphan_zy_map(conn: &Connection) -> RusqliteResult<usize> {
    let n = conn.execute(
        "DELETE FROM _zy_map WHERE (z, y) NOT IN (SELECT DISTINCT zoom_level as z, tile_row as y FROM tiles)",
        [],
    )?;
    Ok(n)
}

pub fn zxyify(conn: &Connection) -> RusqliteResult<Vec<RowsAffected>> {
    let mut affected = vec![];
    if has_zy_map(conn)? {
        let n_removed = deorphan_zy_map(conn)?;
        if n_removed > 0 {
            debug!("Removed {} orphaned rows from _zy_map", n_removed);
            affected.push(RowsAffected {
                type_: AffectedType::Delete,
                table: Some("_zy_map".to_string()),
                count: n_removed,
            });
        }
    } else {
        debug!("Creating _zy_map table");
        create_zy_map_table(conn)?;
    }
    debug!("Creating _zy_map index");
    // don't care if it fails
    create_zy_map_index(conn).ok();
    debug!("Updating _zy_map");
    let zy_rows = update_zy_map(conn)?;
    if zy_rows > 0 {
        debug!("Updated _zy_map with {} rows", zy_rows);
        affected.push(RowsAffected {
            type_: AffectedType::Insert,
            table: Some("_zy_map".to_string()),
            count: zy_rows,
        });
    }
    debug!("Creating zxy view");
    create_zxy_view(conn).ok();
    Ok(affected)
}

pub fn unzxyify(conn: &Connection) -> RusqliteResult<()> {
    if has_zy_map(conn)? {
        drop_zxy_view(conn)?;
        drop_zy_map_index(conn)?;
        drop_zy_map_table(conn)?;
    }
    Ok(())
}
