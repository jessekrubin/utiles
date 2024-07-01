# CHANGELOG

## Unreleased/Future

- lint/copy overhaul

___


## 0.6.1 (2024-07-01)

- Fix calling `utiles.ut_cli` multiple times causing tracing-subscriber crash

___

## 0.6.0 (2024-06-28)

- Upgrade pyo3 to `v0.22.0` -- had to add signatures to all fns with optional args/kwargs
- Update python dev deps
- Added `{bbox}`, `{projwin}`, `{bbox_web}` and `{projwin_web}` format tokens to tile-formatter (those projwins are handy for gdaling)

___

## 0.5.1 (2024-06-19)

- Fixed backpressure issue when `unpyramiding` direcotry to mbtiles; the loading of tiles was happening too fast and could cause memory issues on large tile-pyramids... (this was previously not an issue b/c I would run those jobs on my work machine which had 512gb or ram, but that machine died... RIP titus)
- Write out `metadata.json` when `pyramid-ing` mbtiles to directory if the metadata of the mbtiles does not conatin duplicate keys (which it should not)
- Limit jobs/concurrency when `pyramid-ing` mbtiles to directory to 4 (if not specified by `--jobs`/`-j` option) to prevent nuking machines

___

## 0.5.0 (2024-06-14)

- Moved metadata structs and tools from `utiles-core` to `utiles`
- `utiles-python`
  - Refactoring and reorganizing of code
  - Fixed comparison of LngLat obj
  - Using rust lib for `python -m utiles` file size formatting
- `clippy::pedantic`
  - Many changes on the road to `clippy::pedantic`
  - `utiles-core` is almost fully pedantic
  - `utiles` is becoming pedantic
  - `utiles-python` is not very pedantic yet
- Testing:
  - More tests added (mostly testing w/ python)
  - Added test mbtiles file `osm-standard.0z4.mbtiles`
- fmt-str command and option `--fmt` added `tiles` command; allows string formatting of json-tiles:

```
Format json-tiles format-string
fmt-tokens:
   `{json_arr}`/`{json}`  -> [x, y, z]
   `{json_obj}`/`{obj}`   -> {x: x, y: y, z: z}
   `{quadkey}`/`{qk}`     -> quadkey string
   `{pmtileid}`/`{pmid}`  -> pmtile-id
   `{x}`                  -> x tile coord
   `{y}`                  -> y tile coord
   `{z}`                  -> z/zoom level
   `{-y}`/`{yup}`         -> y tile coord flipped/tms
   `{zxy}`                -> z/x/y
   

Example:
   > echo "[486, 332, 10]" | utiles fmtstr
   [486, 332, 10]
   > echo "[486, 332, 10]" | utiles fmtstr --fmt "{x},{y},{z}"
   486,332,10
   > echo "[486, 332, 10]" | utiles fmt --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column = {x} AND tile_row = {y};"
   SELECT * FROM tiles WHERE zoom_level = 10 AND tile_column = 486 AND tile_row = 332;
```

___

## 0.4.1 (2024-04-04)

- Fixed problem with python tile `__richcmp__` not handling invalid tiles and non-tile-like objs

## 0.4.0 (2024-03-28)

- Updated to pyo3 `v0.21.0`
- Cli help messages cleaned up
- General spring cleaning!
- Hid the `utiles tilejson` cli alias `trader-joes`

___

## 0.3.1 (2024-01-30)

- Minor bug fixes

## 0.3.0 (2024-01-16)

- Expanded utiles cli with several more commands

___ 

## 0.2.0 (2023-11-10)

- Converted cli to rust as an excerise in learning clap
- Moved old click cli to `utiles._legacy.cli`
- Added tilejson/tj command to rust cli to write out tilejson files for mbtiles
- Added meta command to rust cli to write out json of metadata table for mbtiles

___

## 0.1.0 (2023-10-27)

- Drop python 3.7 (was good knowing you)
- Update pyo3 to 0.20.0
- Added rasterio/rio entry points ('utiles' and 'ut' alias bc why type `rio utiles` over `rio ut`)

___

## 0.0.2

- Added `__len__` to TilesGenerator for pbars
