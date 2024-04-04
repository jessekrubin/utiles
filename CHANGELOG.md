# 0.4.1 (2024-04-04)

 - Fixed problem with python tile `__richcmp__` not handling invalid tiles and non-tile-like objs

# 0.4.0 (2024-03-28)

 - Updated to pyo3 `v0.21.0`
 - Cli help messages cleaned up
 - General spring cleaning!
 - Hid the `utiles tilejson` cli alias `trader-joes`

# 0.3.1 (2024-01-30)

 - Minor bug fixes

# 0.3.0 (2024-01-16)

 - Expanded utiles cli with several more commands

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
