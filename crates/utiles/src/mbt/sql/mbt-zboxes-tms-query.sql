-- flip with: (1 << zoom_level - 1 - tile_row)
SELECT
    zoom_level,
    COUNT(*) AS ntiles,
    MIN(tile_row) AS min_tile_row,
    MAX(tile_row) AS max_tile_row,
    MIN(tile_column) AS minx,
    MAX(tile_column) AS maxx,
FROM
    tiles
GROUP BY
    zoom_level;
