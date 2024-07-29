-- Detect if mbtiles file is created by planetiler
--
-- Look for:
--     - 'tiles_shallow' table
--     - `tiles_shallow.tile_data_id` column as integer
--     - 'tiles_data' table
--     - `tiles_data.tile_data_id` column as integer
--
-- A (2024-06-28) planetiler created mbtiles '.schema' looks like:
-- ```sql
-- CREATE TABLE metadata (name text, value text);
-- CREATE UNIQUE INDEX name on metadata (name);
-- CREATE TABLE tiles_shallow (
--   zoom_level integer,
--   tile_column integer,
--   tile_row integer,
--   tile_data_id integer
--   , primary key(zoom_level,tile_column,tile_row)
-- ) without rowid
-- ;
-- CREATE TABLE tiles_data (
--   tile_data_id integer primary key,
--   tile_data blob
-- );
-- CREATE VIEW tiles AS
-- select
--   tiles_shallow.zoom_level as zoom_level,
--   tiles_shallow.tile_column as tile_column,
--   tiles_shallow.tile_row as tile_row,
--   tiles_data.tile_data as tile_data
-- from tiles_shallow
-- join tiles_data on tiles_shallow.tile_data_id = tiles_data.tile_data_id
-- /* tiles(zoom_level,tile_column,tile_row,tile_data) */;
-- ```

SELECT (
     -- Has a 'map' table
     SELECT COUNT(*) = 1
     FROM sqlite_schema
     WHERE name = 'tiles_shallow'
       AND type = 'table'
     --
 ) AND (
     SELECT COUNT(*) = 4
     FROM pragma_table_info('tiles_shallow')
     WHERE ((name = 'zoom_level' AND type = 'INTEGER')
         OR (name = 'tile_column' AND type = 'INTEGER')
         OR (name = 'tile_row' AND type = 'INTEGER')
         OR (name = 'tile_data_id' AND type = 'INTEGER'))
     --
 ) AND (
     -- Has a 'images' table
     SELECT COUNT(*) = 1
     FROM sqlite_schema
     WHERE name = 'tiles_data'
       AND type = 'table'
     --
 ) AND (
     -- 'images' table's columns and their types are as expected:
     -- 3 columns (tile_id, tile_data).
     -- The order is not important
     SELECT COUNT(*) = 2
        FROM pragma_table_info('tiles_data')
        WHERE ((name = 'tile_data_id' AND type = 'INTEGER')
            OR (name = 'tile_data' AND type = 'BLOB'))
 ) AS is_planetiler_mbtiles;
