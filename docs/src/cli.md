# cli

The utiles cli can be installed via cargo or pip; the python library bundles
a version of the cli that can be run just as `utiles` and/or `python -m utiles`.

The cli is a collection of commands for streaming tiles in text format (excellent for piping) and working with mbtiles/tile-db files.

All of the `mercantile` cli commands are available, and `burn` and `edges` from `supermercado` are also available.


___

## What about the `mbtiles` cli in the martin repo? [2025-01-14]

The `mbtiles` cli is very good and has significant overlap with the `utiles` cli. 

(imo) The most intriguing feature of the `mbtiles` cli is the `diff`/`patch` commands
that allow for comparing and updating mbtiles files; `utiles` does not have this feature.

`utiles` does have a streaming mode copy command that allows copying tiles from one mbtiles db to another without using the `sqlite-ATTACH` command; this is in my testing and experience less crash-prone and much faster on super large tile-dbs.

The cli commands in `utiles` that interact with mbtiles are more focused `raster` format `mbtiles` files.


### `agg-hash`


`utiles` adopted the standardized way of calculating an `aggregate-tiles-hash` for an mbtiles file, and builds on it to allow for hash-algorithms aside from `md5`. The cli command also allows for selecting zoom-levels, and bounding-boxes to calculate the hash over.

The default hashing alg is `md5` to match the martin `mbtiles` cli; but may change if I get around to updating some of our (dgi's) very large datasets; 
`xxh64` is generally very fast.


```bash
> utiles agg-hash -z 2 osm-standard.z0z4.mbtiles
{
  "hash_type": "md5",
  "hash": "F211FF7D9FF917B58808302E0AAE82FF",
  "ntiles": 16,
  "dt": {
    "secs": 0,
    "nanos": 709600
  }
}

> utiles agg-hash osm-standard.z0z4.mbtiles --hash md5
{
  "hash_type": "md5",
  "hash": "3A9279283D4D6B5B12362E3A76AF7201",
  "ntiles": 341,
  "dt": {
    "secs": 0,
    "nanos": 10092400
  }
}

> utiles agg-hash osm-standard.z0z4.mbtiles
{
  "hash_type": "md5",
  "hash": "3A9279283D4D6B5B12362E3A76AF7201",
  "ntiles": 341,
  "dt": {
    "secs": 0,
    "nanos": 10108300
  }
}
```

## `copy`



```bash
> utiles tj osm-standard.z0z4.mbtiles
{
  "tilejson": "3.0.0",
  "tiles": [],
  "bounds": [
    -180.0,
    -85.05113,
    180.0,
    85.05113
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm standard png tiles 256",
  "maxzoom": 4,
  "minzoom": 0,
  "name": "osm-standard",
  "format": "png",
  "type": "overlay"
}

> utiles copy osm-standard.z0z4.mbtiles osm-standard.z4.mbtiles -z 4
2025-01-14T22:17:07.113799Z  INFO utiles::copy: copy-config-json: {
  "src": "osm-standard.z0z4.mbtiles",
  "dst": "osm-standard.z4.mbtiles",
  "zset": 16,
  "zooms": [
    4
  ],
  "bboxes": null,
  "bounds_string": null,
  "verbose": true,
  "dryrun": false,
  "force": false,
  "jobs": null,
  "istrat": "None",
  "dst_type": null,
  "hash": null,
  "stream": false
}
2025-01-14T22:17:07.114507Z  WARN utiles::copy::pasta: mbtiles-2-mbtiles copy is a WIP
2025-01-14T22:17:07.116782Z  INFO utiles::copy::pasta: dst_db_type_if_new: Some(Flat)
2025-01-14T22:17:07.154346Z  INFO utiles::copy::pasta: Copying from "osm-standard.z0z4.mbtiles" (flat) -> "osm-standard.z4.mbtiles" flat
2025-01-14T22:17:07.154803Z  INFO utiles::copy::pasta: Copying tiles: "osm-standard.z0z4.mbtiles" -> "osm-standard.z4.mbtiles"
2025-01-14T22:17:07.165062Z  INFO utiles::copy::pasta: Copied 256 tiles from "osm-standard.z0z4.mbtiles" -> "osm-standard.z4.mbtiles" in 10.0263ms

> utiles tj osm-standard.z4.mbtiles
{
  "tilejson": "3.0.0",
  "tiles": [],
  "bounds": [
    -180.0,
    -85.05113,
    180.0,
    85.05113
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm standard png tiles 256",
  "maxzoom": 4,
  "minzoom": 4,
  "name": "osm-standard",
  "dbtype": "flat",
  "format": "png",
  "type": "overlay"
}

```
