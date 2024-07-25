SELECT
    zoom_level,
    ntiles,
    min_x,
    max_x,
    ((1 << zoom_level) - 1 - max_tile_row) AS min_y,
    ((1 << zoom_level) - 1 - min_tile_row) AS max_y
FROM (
    SELECT
        zoom_level,
        COUNT(*) AS ntiles,
        MIN(tile_row) AS min_tile_row,
        MAX(tile_row) AS max_tile_row,
        MIN(tile_column) AS min_x,
        MAX(tile_column) AS max_x
    FROM
        tiles
    GROUP BY
        zoom_level
) sub;
