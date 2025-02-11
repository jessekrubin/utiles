# Definitions

**yup:** in the context of web-map-tiles `yup` means "y-up"; z/x/y tile servers serve tiles with `y-down` coordinates, while `mbtiles` and `geopackage` files store tiles as `y-up`. Converting between `y` and `yup` is done by `yup = 2^z - y - 1` where `z` is the zoom level. Converting in a `sqlite` database can be done like this:

```sql
SELECT zoom_level AS z,
       tile_column AS x,
       (2^zoom_level - tile_row - 1) AS y, 
       tile_row AS yup,
FROM tiles;
```

**zoom-level:** Map zoom level 0 - 30

**tile:** A tile is a square image of a specific zoom level, typically 256x256 pixels. Tiles are used to display map data at different zoom levels.

**zbox:** Utiles name for a tile-bounding-box at a zoom level - defines zoom, min-x, min-y, max-x, max-y for querying tiles

**tilejson:** A JSON object that describes a tile set, including its name, description, and the URL template for accessing tiles. TileJSON is used to provide metadata about a tile set.

**textiles:** utiles name for an ndjson/jsonl file containing tiles as arrays or objects (e.g. `[x, y, z]` or `{x: x, y: y, z: z}`)


