# CHANGELOG

## TODO

- Docs/readme update

___

## 0.7.0 (2025-01-09)

- **NO UNWRAPPING!** there is no unwrapping in utiles!
- updated `thiserror` to v2
- remove `gil-refs` pyo3 feature from python lib
- `pyo3` v0.23.x
- supermercado compatible `edges` and `burn` cli commands added as well a s rust
  lib functions (obviously)
- `shapes` and `feature` functions for tile produce polygons that now DO follow
  the right-hand-rule; exterior rings are clockwise
- `simplify`/`merge` function(s) optimized for faster merging and WAY less
  memory usage
- Parent methods now returns `None` if `z<=0`
- New command(s):
  - `agg-hash` command that computes the `agg-tiles-hash` of a tiles-db as
    standardized by the martin/maplibre team (this supports more hash-types;
    `xxh3` appears to be the fastest and what utiles will likely default to if
    not `xxh64`)
  - `commands` list all available commands (including hidden/dev/unimplemented
    commands)
  - `enumerate` list xyz tiles in db(s) similar to `tippecanoe-enumerate`; for
    tippecanoe compatibility use `--tippecanoe`/`-t`
  - `sqlite`/`db` sub-command group with `vac`/`analyze` commands and will
    likely contain future pure sqlite util(e)s... these could totally be shell
    scripts, but they're nice to have on das-windows
    - `header`/`head` command that prints the json of a sqlite db header (which
      has come in handy for weird dbs that use old code to write out sqlite dbs
      (yes I have seen this))
    - `vac`/`vacuum` command that vacuums a sqlite db (optionally into a new db)
    - `analyze` command that analyzes a sqlite db (basically the same as doing
      `sqlite3 database.sqlite "PRAGMA analyze;"`)
- `copy` and `touch`
  - Now supports `flat`/`norm` (normalized)/`hash` (flat-with-hash) formats as
    standardized by the martin/maplibre people. Should also work with
    non-martin-conforming mbtiles schemas (appears to for me)
- Dev/hidden commands:
  - `webpify` command that converts all non-webp raster-tiles to webp (lossless
    only due to image-crate not supporting lossy encoding...)
  - `oxipng` command that optimizes png(s) in mbtiles db(s) using `oxipng` crate
- figured out how to make async tile-stream(s)
- Removed even more `unwrap` usages
- lager/logging/tracing is reloadable from python; using the new fancy `std::sync::LazyLock` (yay)
- lint/copy overhaul
- Added `--page-size` to vacuum command
- Using `json-patch` for metadata updates
- Allow setting metadata value(s) from file if no value is provided (`-`/`--`)
  for stdin
- Figured out how to put the caller-site (eg `pyo3` in the cli help so you
  (likely me) can tell which utiles you are calling)
- python:
  - Added `TileFmts` string formatter object
  - `Lager` singleton that can toggle tracing format and level (WIP)

---

## 0.6.1 (2024-07-01)

- Fix calling `utiles.ut_cli` multiple times causing tracing-subscriber crash

---

## 0.6.0 (2024-06-28)

- Upgrade pyo3 to `v0.22.0` -- had to add signatures to all fns with optional
  args/kwargs
- Update python dev deps
- Added `{bbox}`, `{projwin}`, `{bbox_web}` and `{projwin_web}` format tokens to
  tile-formatter (those projwins are handy for gdaling)

---

## 0.5.1 (2024-06-19)

- Fixed backpressure issue when `unpyramiding` directory to mbtiles; the loading
  of tiles was happening too fast and could cause memory issues on large
  tile-pyramids... (this was previously not an issue b/c I would run those jobs
  on my work machine which had 512gb or ram, but that machine died... RIP titus)
- Write out `metadata.json` when `pyramid-ing` mbtiles to directory if the
  metadata of the mbtiles does not contain duplicate keys (which it should not)
- Limit jobs/concurrency when `pyramid-ing` mbtiles to directory to 4 (if not
  specified by `--jobs`/`-j` option) to prevent nuking machines

---

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
- fmt-str command and option `--fmt` added `tiles` command; allows string
  formatting of json-tiles:

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

---

## 0.4.1 (2024-04-04)

- Fixed problem with python tile `__richcmp__` not handling invalid tiles and
  non-tile-like objs

## 0.4.0 (2024-03-28)

- Updated to pyo3 `v0.21.0`
- Cli help messages cleaned up
- General spring cleaning!
- Hid the `utiles tilejson` cli alias `trader-joes`

---

## 0.3.1 (2024-01-30)

- Minor bug fixes

## 0.3.0 (2024-01-16)

- Expanded utiles cli with several more commands

---

## 0.2.0 (2023-11-10)

- Converted cli to rust as an exercise in learning clap
- Moved old click cli to `utiles._legacy.cli`
- Added tilejson/tj command to rust cli to write out tilejson files for mbtiles
- Added meta command to rust cli to write out json of metadata table for mbtiles

---

## 0.1.0 (2023-10-27)

- Drop python 3.7 (was good knowing you)
- Update pyo3 to 0.20.0
- Added rasterio/rio entry points ('utiles' and 'ut' alias bc why type
  `rio utiles` over `rio ut`)

---

## 0.0.2

- Added `__len__` to TilesGenerator for pbars
