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
     -- 2 columns (tile_id, tile_data).
     -- The order is not important
     SELECT COUNT(*) = 2
     FROM pragma_table_info('images')
     WHERE ((name = 'tile_id' AND type = 'TEXT')
         OR (name = 'tile_data' AND type = 'BLOB'))
     --
 ) AS is_norm_mbtiles;