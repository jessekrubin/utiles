-- Detect if mbtiles file is created by tippecanoe
--
-- Big distinction is that a tippecannoe mbtiles file has 'zoom_level' column
-- in 'images' table
--
-- A (2024-06-11) tippecanoe created mbtiles schema '.dump' looks like:
-- ```sql
-- CREATE TABLE metadata (name text, value text);
-- CREATE UNIQUE INDEX name on metadata (name);
-- CREATE TABLE map (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_id TEXT);
-- CREATE UNIQUE INDEX map_index ON map (zoom_level, tile_column, tile_row);
-- CREATE TABLE images (zoom_level integer, tile_data blob, tile_id text);
-- CREATE UNIQUE INDEX images_id ON images (zoom_level, tile_id);
-- CREATE VIEW tiles AS SELECT map.zoom_level AS zoom_level, map.tile_column AS tile_column, map.tile_row AS tile_row, images.tile_data AS tile_data FROM map JOIN images ON images.tile_id = map.tile_id and images.zoom_level = map.zoom_level
-- ```
SELECT (
     -- Has a 'map' table
     SELECT COUNT(*) = 1
     FROM sqlite_schema
     WHERE name = 'map'
       AND type = 'table'
     --
 ) AND (
     -- 'map' table's columns and their types are as expected:
     -- 4 columns (zoom_level, tile_column, tile_row, tile_id).
     -- The order is not important
     SELECT COUNT(*) = 4
     FROM pragma_table_info('map')
     WHERE ((name = 'zoom_level' AND type = 'INTEGER')
         OR (name = 'tile_column' AND type = 'INTEGER')
         OR (name = 'tile_row' AND type = 'INTEGER')
         OR (name = 'tile_id' AND type = 'TEXT'))
     --
 ) AND (
     -- Has a 'images' table
     SELECT COUNT(*) = 1
     FROM sqlite_schema
     WHERE name = 'images'
       AND type = 'table'
     --
 ) AND (
     -- 'images' table's columns and their types are as expected:
     -- 3 columns (tile_id, tile_data).
     -- The order is not important
     SELECT COUNT(*) = 3
     FROM pragma_table_info('images')
     WHERE ((name = 'tile_id' AND type = 'TEXT')
         OR (name = 'tile_data' AND type = 'BLOB')
         OR (name = 'zoom_level' AND type = 'INTEGER'))
     --
 ) AS is_norm_mbtiles;