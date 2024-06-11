SELECT (
     -- Has a 'tiles' table
     SELECT COUNT(*) = 1
     FROM sqlite_schema
     WHERE name = 'tiles'
       AND type = 'table'
     --
 ) AND (
     -- 'tiles' table's columns and their types are as expected:
     -- 4 columns (zoom_level, tile_column, tile_row, tile_data).
     -- The order is not important
     SELECT COUNT(*) = 4
     FROM pragma_table_info('tiles')
     WHERE ((name = 'zoom_level' AND type = 'INTEGER')
         OR (name = 'tile_column' AND type = 'INTEGER')
         OR (name = 'tile_row' AND type = 'INTEGER')
         OR (name = 'tile_data' AND type = 'BLOB'))
     --
 ) as is_flat_mbtiles;