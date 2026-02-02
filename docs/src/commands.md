# utiles CLI commands

## Table of contents

| Command | Description |
| ------- | ----------- |
| [about](#about) | Echo info about utiles |
| [agg-hash](#agg-hash) | Aggregate tile hashes for tiles-db |
| [bounding-tile](#bounding-tile) | Echo bounding tile at zoom for bbox / geojson |
| [burn](#burn) | Burn tiles from `GeoJSON` stream at zoom level (tile coverage) |
| [children](#children) | Echo children tiles of input tiles |
| [commands](#commands) | list all commands |
| [copy](#copy) | Copy tiles from src -> dst |
| [dbcontains](#dbcontains) | Determine if mbtiles contains a latlong |
| [dev](#dev) | Development/Playground command (hidden) |
| [edges](#edges) | Echo edge tiles from stream of xyz tiles |
| [enumerate](#enumerate) | Enumerate tiles db |
| [fmtstr](#fmtstr) | Format json-tiles `[x, y, z]` tiles w/ format-string |
| [info](#info) | Echo mbtiles info/stats |
| [lint](#lint) | Lint mbtiles file(s) (wip) |
| [merge](#merge) | Merge tiles from stream removing parent tiles if children are present |
| [metadata](#metadata) | Echo metadata (table) as json arr/obj |
| [metadata-set](#metadata-set) | Set metadata key/value or from `json` file if key is fspath |
| [neighbors](#neighbors) | Echo the neighbor tiles for input tiles |
| [optimize](#optimize) | Optimize tiles-db |
| [parent](#parent) | Echo parent tile of input tiles |
| [pmtileid](#pmtileid) | Converts tile(s) to/from pmtile-id/[x, y, z] |
| [quadkey](#quadkey) | Converts tiles to/from quadkey/[x, y, z] |
| [rimraf](#rimraf) | rm-rf dirpath |
| [serve](#serve) | utiles server (wip) |
| [shapes](#shapes) | Echo tiles as `GeoJSON` feature collections/sequences |
| [sqlite](#sqlite) | sqlite utils/cmds |
| [tilejson](#tilejson) | Echo the `tile.json` for mbtiles file |
| [tiles](#tiles) | Echo tiles at zoom intersecting geojson bbox / feature / collection |
| [touch](#touch) | Create new mbtiles db w/ schema |
| [update](#update) | Update mbtiles db |
| [vacuum](#vacuum) | vacuum sqlite db inplace/into |
| [webpify](#webpify) | Convert raster mbtiles to webp format |
| [zxyify](#zxyify) | zxyify/unzxyify tiles-db |

___

## about

```bash
Echo info about utiles

Usage: utiles about [OPTIONS]

Options:
      --debug     debug mode (print/log more)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## agg-hash

```bash
Aggregate tile hashes for tiles-db

Usage: utiles agg-hash [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  sqlite filepath

Options:
      --debug              debug mode (print/log more)
  -m, --min                compact/minified json (default: false)
      --bbox <BBOX>        bbox(es) (west, south, east, north)
      --trace              trace mode (print/log EVEN more)
      --log-json           format log as NDJSON
  -z, --zoom <ZOOM>        Zoom level (0-30)
      --minzoom <MINZOOM>  min zoom level (0-30)
      --maxzoom <MAXZOOM>  max zoom level (0-30)
      --hash <HASH>        hash to use for blob-id if copying to normal/hash db type [possible
                           values: md5, fnv1a, xxh32, xxh64, xxh3-64, xxh3-128]
  -h, --help               Print help

```

___

## bounding-tile

```bash
Echo the Web Mercator tile at ZOOM level bounding `GeoJSON` [west, south,
east, north] bounding boxes, features, or collections read from stdin.

Input may be a compact newline-delimited sequences of JSON or a
pretty-printed ASCII RS-delimited sequence of JSON (like
<https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Examples:

  \> echo "[-105.05, 39.95, -105, 40]" | utiles bounding-tile
  [426, 775, 11]

Usage: utiles bounding-tile [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## burn

```bash
Burn tiles from `GeoJSON` stream at zoom level (tile coverage)

Usage: utiles burn [OPTIONS] <ZOOM> [INPUT]

Arguments:
  <ZOOM>
          Zoom level (0-30)

  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## children

```bash
Echo children tiles of input tiles

Input may be a compact newline-delimited sequences of JSON or a
pretty-printed ASCII RS-delimited sequence of JSON (like
<https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Example:

  \> echo "[486, 332, 10]" | utiles children
  [972, 664, 11]

Usage: utiles children [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

      --depth <DEPTH>
          [default: 1]

  -h, --help
          Print help (see a summary with '-h')

```

___

## commands

```bash
list all commands

Usage: utiles commands [OPTIONS]

Options:
      --debug     debug mode (print/log more)
  -f, --full      
  -t, --table     
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## copy

```bash
Copy tiles from src -> dst

Usage: utiles copy [OPTIONS] <SRC> <DST>

Arguments:
  <SRC>  source dataset fspath (mbtiles, dirpath)
  <DST>  destination dataset fspath (mbtiles, dirpath)

Options:
      --debug                debug mode (print/log more)
  -n, --dryrun               dryrun (don't actually copy)
  -f, --force                force overwrite dst
      --trace                trace mode (print/log EVEN more)
      --log-json             format log as NDJSON
  -z, --zoom <ZOOM>          Zoom level (0-30)
      --minzoom <MINZOOM>    min zoom level (0-30)
      --maxzoom <MAXZOOM>    max zoom level (0-30)
      --bbox <BBOX>          bbox (west, south, east, north)
  -c, --conflict <CONFLICT>  conflict strategy when copying tiles [default: undefined] [possible
                             values: undefined, ignore, replace, abort, fail]
      --dst-type <DST_TYPE>  db-type (default: src type) [possible values: flat, hash, norm]
      --hash <HASH>          hash to use for blob-id if copying to normal/hash db type [possible
                             values: md5, fnv1a, xxh32, xxh64, xxh3-64, xxh3-128]
  -j, --jobs <JOBS>          n-jobs ~ 0=ncpus (default: max(4, ncpus))
  -h, --help                 Print help

```

___

## dbcontains

```bash
Determine if mbtiles contains a latlong

Usage: utiles dbcontains [OPTIONS] <FILEPATH> <LNGLAT>

Arguments:
  <FILEPATH>  mbtiles filepath
  <LNGLAT>    lat/long

Options:
      --debug     debug mode (print/log more)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## dev

```bash
Development/Playground command (hidden)

Usage: utiles dev [OPTIONS] [FSPATH]

Arguments:
  [FSPATH]  

Options:
      --debug     debug mode (print/log more)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## edges

```bash
Echo edge tiles from stream of xyz tiles

Usage: utiles edges [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --wrapx
          

      --seq
          Write tiles as RS-delimited JSON sequence

      --trace
          trace mode (print/log EVEN more)

      --log-json
          format log as NDJSON

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

  -h, --help
          Print help (see a summary with '-h')

```

___

## enumerate

```bash
Enumerate tiles db

Usage: utiles enumerate [OPTIONS] <FSPATHS>...

Arguments:
  <FSPATHS>...  

Options:
      --bbox <BBOX>        bbox(es) (west, south, east, north)
      --debug              debug mode (print/log more)
      --trace              trace mode (print/log EVEN more)
  -z, --zoom <ZOOM>        Zoom level (0-30)
      --log-json           format log as NDJSON
      --minzoom <MINZOOM>  min zoom level (0-30)
      --maxzoom <MAXZOOM>  max zoom level (0-30)
  -t, --tippecanoe         tippecanoe-enumerate like output '{relpath} {x} {y} {z}'
  -h, --help               Print help

```

___

## fmtstr

```bash
Format json-tiles `[x, y, z]` tiles w/ format-string

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
    ```
    \> echo "[486, 332, 10]" | utiles fmtstr
    [486, 332, 10]
    \> echo "[486, 332, 10]" | utiles fmtstr --fmt "{x},{y},{z}"
    486,332,10
    \> echo "[486, 332, 10]" | utiles fmt --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND
    tile_column = {x} AND tile_row = {y};"
    SELECT * FROM tiles WHERE zoom_level = 10 AND tile_column = 486 AND tile_row = 332;
    ```

Usage: utiles fmtstr [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## info

```bash
Echo mbtiles info/stats

Usage: utiles info [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  sqlite filepath

Options:
      --debug       debug mode (print/log more)
  -m, --min         compact/minified json (default: false)
      --full        
      --trace       trace mode (print/log EVEN more)
      --log-json    format log as NDJSON
  -s, --statistics  [aliases: stats]
  -h, --help        Print help

```

___

## lint

```bash
Lint mbtiles file(s) (wip)

Usage: utiles lint [OPTIONS] <FSPATHS>...

Arguments:
  <FSPATHS>...  filepath(s) or dirpath(s)

Options:
      --debug     debug mode (print/log more)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## merge

```bash
Merge tiles from stream removing parent tiles if children are present

Usage: utiles merge [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

  -Z, --minzoom <MINZOOM>
          min zoom level (0-30) to merge to
          
          [default: 0]

  -s, --sort
          

      --trace
          trace mode (print/log EVEN more)

      --log-json
          format log as NDJSON

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

  -h, --help
          Print help (see a summary with '-h')

```

___

## metadata

```bash
Echo metadata (table) as json arr/obj

Usage: utiles metadata [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  sqlite filepath

Options:
      --debug     debug mode (print/log more)
  -m, --min       compact/minified json (default: false)
      --obj       Output as json object not array
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
      --raw       Output as json string for values (default: false)
  -h, --help      Print help

```

___

## metadata-set

```bash
Set metadata key/value or from `json` file if key is fspath

Usage: utiles metadata-set [OPTIONS] <FILEPATH> <KEY/FSPATH> [VALUE]

Arguments:
  <FILEPATH>    sqlite filepath
  <KEY/FSPATH>  key or json-fspath
  [VALUE]       value

Options:
      --debug     debug mode (print/log more)
  -m, --min       compact/minified json (default: false)
  -n, --dryrun    dryrun (don't actually set)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## neighbors

```bash
Echo the neighbor tiles for input tiles

Input may be a compact newline-delimited sequences of JSON or a pretty-printed ASCII RS-delimited
sequence of JSON (like <https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Usage: utiles neighbors [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## optimize

```bash
Optimize tiles-db

Usage: utiles optimize [OPTIONS] <FILEPATH> <DST>

Arguments:
  <FILEPATH>  sqlite filepath
  <DST>       destination dataset fspath (mbtiles, dirpath)

Options:
      --debug     debug mode (print/log more)
  -m, --min       compact/minified json (default: false)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## parent

```bash
Echo parent tile of input tiles

Usage: utiles parent [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

      --depth <DEPTH>
          [default: 1]

  -h, --help
          Print help (see a summary with '-h')

```

___

## pmtileid

```bash
Converts tile(s) to/from pmtile-id/[x, y, z]

Input may be a compact newline-delimited sequences of JSON or a
pretty-printed ASCII RS-delimited sequence of JSON (like
<https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Examples:

  \> echo "[486, 332, 10]" | utiles pmtileid
  506307
  \> echo "506307" | utiles pmtileid
  [486, 332, 10]
  \> utiles pmtileid 506307
  [486, 332, 10]

Usage: utiles pmtileid [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## quadkey

```bash
Converts tiles to/from quadkey/[x, y, z]

Input may be a compact newline-delimited sequences of JSON or a
pretty-printed ASCII RS-delimited sequence of JSON (like
<https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Examples:

  \> echo "[486, 332, 10]" | utiles quadkey
  0313102310
  \> echo "0313102310" | utiles quadkey
  [486, 332, 10]
  \> utiles quadkey 0313102310
  [486, 332, 10]

Usage: utiles quadkey [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## rimraf

```bash
rm-rf dirpath

Usage: utiles rimraf [OPTIONS] <DIRPATH>

Arguments:
  <DIRPATH>  dirpath to nuke

Options:
      --debug     debug mode (print/log more)
      --size      collect and print file sizes
  -n, --dryrun    dryrun (don't actually rm)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
      --verbose   
  -h, --help      Print help

```

___

## serve

```bash
utiles server (wip)

Usage: utiles serve [OPTIONS] [FSPATHS]...

Arguments:
  [FSPATHS]...  Filesystem paths to serve from

Options:
      --debug        debug mode (print/log more)
  -p, --port <PORT>  Port to server on [default: 3333]
  -H, --host <HOST>  Host bind address [default: 0.0.0.0]
      --trace        trace mode (print/log EVEN more)
      --log-json     format log as NDJSON
  -s, --strict       strict mode (default: false)
  -h, --help         Print help

```

___

## shapes

```bash
Echo tiles as `GeoJSON` feature collections/sequences

Input may be a compact newline-delimited sequences of JSON or a pretty-printed ASCII RS-delimited
sequence of JSON (like <https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Example:

\> echo "[486, 332, 10]" | utiles shapes --precision 4 --bbox [-9.1406, 53.1204, -8.7891, 53.3309]

Usage: utiles shapes [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          

      --precision <PRECISION>
          Decimal precision of coordinates

      --trace
          trace mode (print/log EVEN more)

      --geographic
          Output in geographic coordinates (the default)

      --log-json
          format log as NDJSON

      --mercator
          Output in Web Mercator coordinates

      --feature
          Output as a `GeoJSON` feature collections

      --bbox
          Output in Web Mercator coordinates

      --collect
          Output as a `GeoJSON` feature collections

      --extents
          Write shape extents as ws-separated strings (default is False)

      --buffer <BUFFER>
          Shift shape x and y values by a constant number

  -h, --help
          Print help (see a summary with '-h')

```

___

## sqlite

```bash
sqlite utils/cmds

Usage: utiles sqlite [OPTIONS] <COMMAND>

Commands:
  analyze  Analyze sqlite db
  header   Dump sqlite db header
  vacuum   vacuum sqlite db inplace/into
  help     Print this message or the help of the given subcommand(s)

Options:
      --debug     debug mode (print/log more)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## tilejson

```bash
Echo the `tile.json` for mbtiles file

Usage: utiles tilejson [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  sqlite filepath

Options:
      --debug      debug mode (print/log more)
  -m, --min        compact/minified json (default: false)
  -t, --tilestats  include tilestats
      --trace      trace mode (print/log EVEN more)
      --log-json   format log as NDJSON
  -h, --help       Print help

```

___

## tiles

```bash
Echos web-mercator tiles at zoom level intersecting given geojson-bbox [west, south,
east, north], geojson-features, or geojson-collections read from stdin.

Output format is a JSON `[x, y, z]` array by default; use --obj to output a
JSON object `{x: x, y: y, z: z}`.

bbox shorthands (case-insensitive):
    "*"  | "world"     => [-180, -85.0511, 180, 85.0511]
    "n"  | "north"     => [-180, 0, 180, 85.0511]
    "s"  | "south"     => [-180, -85.0511, 180, 0]
    "e"  | "east"      => [0, -85.0511, 180, 85.0511]
    "w"  | "west"      => [-180, -85.0511, 0, 85.0511]
    "ne" | "northeast" => [0, 0, 180, 85.0511]
    "se" | "southeast" => [0, -85.0511, 180, 0]
    "nw" | "northwest" => [-180, 0, 0, 85.0511]
    "sw" | "southwest" => [-180, -85.0511, 0, 0]

Input may be a compact newline-delimited sequences of JSON or a
pretty-printed ASCII RS-delimited sequence of JSON (like
<https://tools.ietf.org/html/rfc8142> and
<https://tools.ietf.org/html/rfc7159>).

Example:

  \\> echo "[-105.05, 39.95, -105, 40]" | utiles tiles 12
  [852, 1550, 12]
  [852, 1551, 12]
  [853, 1550, 12]
  [853, 1551, 12]
  \> utiles tiles 12 "[-105.05, 39.95, -105, 40]"
  [852, 1550, 12]
  [852, 1551, 12]
  [853, 1550, 12]
  [853, 1551, 12]

Usage: utiles tiles [OPTIONS] <ZOOM> [INPUT]

Arguments:
  <ZOOM>
          Zoom level (0-30)

  [INPUT]
          

Options:
      --debug
          debug mode (print/log more)

      --seq
          Write tiles as RS-delimited JSON sequence

      --obj
          Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)

      --trace
          trace mode (print/log EVEN more)

  -F, --fmt <FMT>
          Format string for tiles (default: `{json_arr}`)
          
          Example:
              > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
              http://thingy.com/1/0/0.png
              http://thingy.com/1/0/1.png
              http://thingy.com/1/1/0.png
              http://thingy.com/1/1/1.png
              > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column =
              {x} AND tile_row = {-y};"
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
              SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;
          
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
              `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
              `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
              `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
              `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```

___

## touch

```bash
Create new mbtiles db w/ schema

Usage: utiles touch [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  mbtiles filepath

Options:
      --debug                  debug mode (print/log more)
      --page-size <PAGE_SIZE>  page size
      --dbtype <DBTYPE>        db-type (default: flat) [default: flat] [possible values: flat, hash,
                               norm]
      --trace                  trace mode (print/log EVEN more)
      --log-json               format log as NDJSON
  -h, --help                   Print help

```

___

## update

```bash
Update mbtiles db

Usage: utiles update [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  sqlite filepath

Options:
      --debug     debug mode (print/log more)
  -m, --min       compact/minified json (default: false)
  -n, --dryrun    dryrun (don't actually update)
      --trace     trace mode (print/log EVEN more)
      --log-json  format log as NDJSON
  -h, --help      Print help

```

___

## vacuum

```bash
vacuum sqlite db inplace/into

Usage: utiles vacuum [OPTIONS] <FILEPATH> [INTO]

Arguments:
  <FILEPATH>  sqlite filepath
  [INTO]      fspath to vacuum db into

Options:
      --debug                  debug mode (print/log more)
  -m, --min                    compact/minified json (default: false)
  -a, --analyze                Analyze db after vacuum
      --trace                  trace mode (print/log EVEN more)
      --log-json               format log as NDJSON
      --page-size <PAGE_SIZE>  page size to set
  -h, --help                   Print help

```

___

## webpify

```bash
Convert raster mbtiles to webp format

Usage: utiles webpify [OPTIONS] <FILEPATH> <DST>

Arguments:
  <FILEPATH>  sqlite filepath
  <DST>       destination dataset fspath (mbtiles, dirpath)

Options:
      --debug        debug mode (print/log more)
  -m, --min          compact/minified json (default: false)
  -j, --jobs <JOBS>  n-jobs ~ 0=ncpus (default: max(4, ncpus))
      --trace        trace mode (print/log EVEN more)
      --log-json     format log as NDJSON
  -q, --quiet        quiet
  -h, --help         Print help

```

___

## zxyify

```bash
zxyify/unzxyify tiles-db

Adds/removes `z/x/y` table/view for querying tiles not inverted

Usage: utiles zxyify [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>
          sqlite filepath

Options:
      --debug
          debug mode (print/log more)

  -m, --min
          compact/minified json (default: false)

      --rm
          un-zxyify a db

      --trace
          trace mode (print/log EVEN more)

      --log-json
          format log as NDJSON

  -h, --help
          Print help (see a summary with '-h')

```
