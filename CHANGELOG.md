# 0.2.0 (2023-11-10)

 - Converted cli to rust as an excerise in learning clap
 - Moved old click cli to `utiles._legacy.cli`
 - Added tilejson/tj command to rust cli to write out tilejson files for mbtiles
 - Added meta command to rust cli to write out json of metadata table for mbtiles

# 0.1.0 (2023-10-27)

 - Drop python 3.7 (was good knowing you)
 - Update pyo3 to 0.20.0
 - Added rasterio/rio entry points ('utiles' and 'ut' alias bc why type `rio utiles` over `rio ut`)

# 0.0.2

 - Added `__len__` to TilesGenerator for pbars
