pub const MBTILES_METADATA_SQLITE_SCHEMA: &str = "
-- metadata table
CREATE TABLE metadata(name TEXT NOT NULL, value TEXT);
-- unique index on name
CREATE UNIQUE INDEX metadata_index ON metadata (name);
";

pub const MBTILES_FLAT_SQLITE_SCHEMA: &str = "
-- metadata table
CREATE TABLE metadata(name TEXT NOT NULL, value TEXT);
-- unique index on name
CREATE UNIQUE INDEX metadata_index ON metadata (name);

-- tiles table
CREATE TABLE tiles
(
    zoom_level  INTEGER NOT NULL,
    tile_column INTEGER NOT NULL,
    tile_row    INTEGER NOT NULL,
    tile_data   BLOB
);
-- unique index on zoom_level, tile_column, tile_row
CREATE UNIQUE INDEX tile_index ON tiles (zoom_level, tile_column, tile_row);
";

pub const MBTILES_HASH_TEXT_SQLITE_SCHEMA: &str = "
-- metadata table
CREATE TABLE metadata (name  TEXT NOT NULL,value TEXT
);
-- unique index on name
CREATE UNIQUE INDEX metadata_index ON metadata (name);
-- tiles table
CREATE TABLE tiles_with_hash
(
    zoom_level  INTEGER NOT NULL,
    tile_column INTEGER NOT NULL,
    tile_row    INTEGER NOT NULL,
    tile_data   BLOB,
    tile_hash   TEXT
);

CREATE UNIQUE INDEX tiles_with_hash_index on tiles_with_hash (zoom_level, tile_column, tile_row);

CREATE VIEW tiles AS
SELECT zoom_level, tile_column, tile_row, tile_data
FROM tiles_with_hash;
";
