SELECT (
   -- 'tiles_with_hash' table or view columns and their types are as expected:
   -- 5 columns (zoom_level, tile_column, tile_row, tile_data, tile_hash).
   -- The order is not important
   SELECT COUNT(*) = 5
   FROM pragma_table_info('tiles_with_hash')
   WHERE ((name = 'zoom_level' AND type = 'INTEGER')
       OR (name = 'tile_column' AND type = 'INTEGER')
       OR (name = 'tile_row' AND type = 'INTEGER')
       OR (name = 'tile_data' AND type = 'BLOB')
       OR (name = 'tile_hash' AND type = 'TEXT'))
   --
) AND (
  -- Has a 'tiles_with_hash' table
  SELECT COUNT(*) = 1
  FROM sqlite_schema
  WHERE name = 'tiles_with_hash' AND type = 'table'
  --
) as is_hash_mbtiles;
