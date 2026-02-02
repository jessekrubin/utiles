-- metadata table
CREATE TABLE metadata(name TEXT NOT NULL, value TEXT);
-- tiles table
CREATE TABLE tiles
(
    zoom_level  INTEGER NOT NULL,
    tile_column INTEGER NOT NULL,
    tile_row    INTEGER NOT NULL,
    tile_data   BLOB
);
-- unique index on name
CREATE UNIQUE INDEX metadata_index ON metadata (name);
-- unique index on zoom_level, tile_column, tile_row
CREATE UNIQUE INDEX tile_index ON tiles (zoom_level, tile_column, tile_row);
