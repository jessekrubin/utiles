# utiles (python)

[![PyPI](https://img.shields.io/pypi/v/utiles.svg?logo=python&style=flat-square&logoColor=white&color=blue)](https://pypi.org/project/utiles)
![Python Version from PEP 621 TOML](https://img.shields.io/python/required-version-toml?tomlFilePath=https%3A%2F%2Fraw.githubusercontent.com%2Fjessekrubin%2Futiles%2Fmain%2Futiles-pyo3%2Fpyproject.toml&style=flat-square&logo=python&logoColor=white&color=blue)
[![Wheel](https://img.shields.io/pypi/wheel/utiles.svg?style=flat-square)](https://img.shields.io/pypi/wheel/utiles.svg)

`utiles = utils + tiles` OR `utiles = ultra-tiles` depending on the day.

Fast spherical mercator geo/tile util(e)ities.

A mostly drop-in replacement for [mercantile](https://github.com/mapbox/mercantile) written w/ rust, plus several other
util(e)ities.

## Installation

```bash
pip install utiles
uv add utiles
```

## Usage

```python
>>> import utiles as ut
>>> from utiles import Tile, LngLat, LngLatBbox
>>> ut.bounds(1, 1, 1)
LngLatBbox(west=0, south=-85.0511287798066, east=180, north=0)
>>> t = ut.Tile(1, 2, 3)
>>> t
Tile(x=1, y=2, z=3)
>>> t.x, t.y, t.z
(1, 2, 3)
>>> x, y, z = t
>>> (x, y, z)
(1, 2, 3)
>>> list(ut.tiles(*ut.bounds(1, 1, 1), 3))
[Tile(x=4, y=4, z=3), Tile(x=4, y=5, z=3), Tile(x=4, y=6, z=3), Tile(x=4, y=7, z=3), Tile(x=5, y=4, z=3), Tile(x=5, y=5, z=3), Tile(x=5, y=6, z=3), Tile(x=5, y=7, z=3), Tile(x=6, y=4, z=3), Tile(x=6, y=5, z=3), Tile(x=6, y=6, z=3), Tile(x=6, y=7, z=3), Tile(x=7, y=4, z=3), Tile(x=7, y=5, z=3), Tile(x=7, y=6, z=3), Tile(x=7, y=7, z=3)]
>>> t
Tile(x=1, y=2, z=3)
>>> t.parent()
Tile(x=0, y=1, z=2)
>>> t.children()
[Tile(x=2, y=4, z=4), Tile(x=3, y=4, z=4), Tile(x=3, y=5, z=4), Tile(x=2, y=5, z=4)]
>>> t.bounds()
LngLatBbox(west=-135, south=40.97989806962013, east=-90, north=66.51326044311186)
>>> t.ul()
LngLat(lng=-135, lat=66.51326044311186)
>>> t.asdict()
{'x': 1, 'y': 2, 'z': 3}
>>> t.center()
LngLat(lng=-112.5, lat=53.74657925636599)
>>> ~t
Tile(x=1, y=5, z=3)
>>> t.valid()  # check if tile is valid
True
>>> ut.Tile(1000, 1231234124, 2).valid()  # invalid tile
False
>>> t.pmtileid()  # return the pmtileid of the tile
34
>>> ut.Tile.from_pmtileid(34)  # create a tile from pmtileid
Tile(x=1, y=2, z=3)
>>> t.json_arr()  # json-array string
'[1, 2, 3]'
>>> t.json_obj()  # json-object string
'{"x":1,"y":2,"z":3}'
>>> t.fmt_zxy()  # format tile as z/x/y
'3/1/2'
>>> t.fmt_zxy_ext('png')  # format tile as z/x/y.ext
'3/1/2.png'
>>> t == (1, 2, 3)  # compare with tuple
True
>>> t == (1, 2, 2234234)  # compare with tuple
False
```

## About

### Why?

I use mercantile regularly and wished it were a bit more ergonomic, had type annotations, and was faster, but overall
it's a great library.

This was an excuse to learn some more rust as well as pyo3.

**Do I/you REALLY need a rust-port of mercantile?**

I don't know, decide for yourself. `utiles` is certainly faster than `mercantile` for some things (see benchmarks below)

**Is it really a drop in replacement for mercantile?**

Not quite, but it's close. utiles doesn't throw the same exceptions as mercantile, instead it throws `ValueError`'s and
`TypeError`'s.

There might be other differences, but I have been using it instead of mercantile for a bit now and it works pretty
decent, tho I am open to suggestions!

## Benchmarks (WIP)

```
---------------------------------------------------------------------------------------------------- benchmark 'quadkey': 12 tests -----------------------------------------------------------------------------------------------------
Name (time in ns)                                        Min                     Max                  Mean              StdDev                Median                 IQR            Outliers  OPS (Kops/s)            Rounds  Iterations
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
test_quadkey_bench[utiles-(0, 0, 0)]                199.9942 (1.0)       47,100.0021 (8.78)       284.7909 (1.0)      315.1058 (6.70)       299.9950 (1.06)     100.0008 (>1000.0)  966;1164    3,511.3476 (1.0)       38911           1
test_quadkey_bench[utiles-(1, 1, 1)]                252.6316 (1.26)       5,363.1581 (1.0)        293.9171 (1.03)      47.0478 (1.0)        284.2108 (1.0)       10.5264 (>1000.0)2884;35689    3,402.3204 (0.97)     196079          19
test_quadkey_bench[utiles-(1, 0, 1)]                299.9950 (1.50)      86,300.0023 (16.09)      397.2831 (1.39)     383.5726 (8.15)       399.9958 (1.41)       0.0073 (1.0)    1451;22409    2,517.0967 (0.72)      99010           1
test_quadkey_bench[mercantile-(0, 0, 0)]            599.9973 (3.00)      28,200.0037 (5.26)       821.2744 (2.88)     301.0209 (6.40)       799.9988 (2.81)       0.0073 (1.0)     658;21559    1,217.6198 (0.35)      69445           1
test_quadkey_bench[utiles-(1, 40, 7)]               599.9973 (3.00)     136,899.9947 (25.53)      758.0325 (2.66)     676.4311 (14.38)      699.9981 (2.46)       0.0073 (1.0)     565;29079    1,319.2047 (0.38)     108696           1
test_quadkey_bench[utiles-(486, 332, 10)]           749.9999 (3.75)       8,055.0002 (1.50)       838.5705 (2.94)     137.5439 (2.92)       824.9997 (2.90)      23.7496 (>1000.0) 1445;4742    1,192.5056 (0.34)      63695          20
test_quadkey_bench[mercantile-(1, 0, 1)]            799.9988 (4.00)     104,300.0011 (19.45)    1,015.6996 (3.57)     539.0831 (11.46)    1,000.0003 (3.52)       0.0073 (1.0)    1217;51791      984.5431 (0.28)     119048           1
test_quadkey_bench[mercantile-(1, 1, 1)]            799.9988 (4.00)      75,999.9966 (14.17)    1,047.5805 (3.68)     419.8019 (8.92)     1,000.0003 (3.52)     100.0008 (>1000.0) 3366;4074      954.5806 (0.27)     166667           1
test_quadkey_bench[utiles-(486, 332, 20)]         1,299.9953 (6.50)      83,399.9948 (15.55)    1,545.1801 (5.43)     461.2615 (9.80)     1,499.9969 (5.28)     100.0008 (>1000.0)8793;17328      647.1738 (0.18)     163935           1
test_quadkey_bench[mercantile-(1, 40, 7)]         1,599.9976 (8.00)     110,599.9982 (20.62)    1,789.4247 (6.28)     711.1950 (15.12)    1,799.9992 (6.33)     100.0008 (>1000.0) 1599;2703      558.8388 (0.16)     116280           1
test_quadkey_bench[mercantile-(486, 332, 10)]     1,999.9934 (10.00)    117,000.0032 (21.82)    2,353.1110 (8.26)     768.5591 (16.34)    2,300.0030 (8.09)     200.0015 (>1000.0) 1917;2168      424.9693 (0.12)     117648           1
test_quadkey_bench[mercantile-(486, 332, 20)]     3,199.9953 (16.00)     66,100.0013 (12.32)    3,601.3369 (12.65)    567.1348 (12.05)    3,599.9983 (12.67)    100.0080 (>1000.0) 1479;4347      277.6747 (0.08)      97088           1
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

---------------------------------------------------------------------------------------------- benchmark 'tiles': 2 tests ---------------------------------------------------------------------------------------------
Name (time in us)                           Min                   Max                  Mean              StdDev                Median                 IQR            Outliers         OPS            Rounds  Iterations
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
test_tiles_gen_bench[utiles]           239.3000 (1.0)      1,597.3000 (1.0)        308.5684 (1.0)      130.3316 (1.0)        267.2000 (1.0)       16.5000 (1.0)       312;559  3,240.7721 (1.0)        3232           1
test_tiles_gen_bench[mercantile]     1,349.9000 (5.64)     7,159.2000 (4.48)     1,798.2186 (5.83)     779.7610 (5.98)     1,526.7000 (5.71)     149.6250 (9.07)       66;111    556.1059 (0.17)        601           1
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

------------------------------------------------------------------------------------------------------- benchmark 'ul': 12 tests ------------------------------------------------------------------------------------------------------
Name (time in ns)                                   Min                     Max                  Mean                StdDev                Median                 IQR              Outliers  OPS (Kops/s)            Rounds  Iterations
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
test_ul_bench[utiles-(1, 1, 1)]                204.3478 (1.0)        7,160.8697 (1.0)        263.7100 (1.0)        125.3400 (1.00)       221.7392 (1.0)       26.0868 (1.30)    17101;28014    3,792.0436 (1.0)      169492          23
test_ul_bench[utiles-(1, 0, 1)]                229.9999 (1.13)      10,579.9998 (1.48)       273.2589 (1.04)       124.7846 (1.0)        250.0001 (1.13)      20.0002 (1.0)      9266;14360    3,659.5327 (0.97)     188680          20
test_ul_bench[utiles-(1, 40, 7)]               229.9999 (1.13)      42,870.0001 (5.99)       311.4689 (1.18)       188.2129 (1.51)       255.0001 (1.15)      35.0003 (1.75)    16764;39465    3,210.5932 (0.85)     200000          20
test_ul_bench[utiles-(486, 332, 20)]           229.9999 (1.13)      65,699.9997 (9.17)       318.4368 (1.21)       243.5307 (1.95)       259.9998 (1.17)      35.0003 (1.75)    11008;36596    3,140.3404 (0.83)     178572          20
test_ul_bench[utiles-(0, 0, 0)]                299.9950 (1.47)      33,899.9962 (4.73)       349.3773 (1.32)       205.0577 (1.64)       300.0023 (1.35)     100.0008 (5.00)        618;618    2,862.2349 (0.75)      70423           1
test_ul_bench[utiles-(486, 332, 10)]           299.9950 (1.47)      57,999.9978 (8.10)       403.1283 (1.53)       400.2343 (3.21)       399.9958 (1.80)     100.0008 (5.00)     2013;20449    2,480.5998 (0.65)     192308           1
test_ul_bench[mercantile-(0, 0, 0)]            999.9931 (4.89)     206,099.9977 (28.78)    1,296.5665 (4.92)     1,201.7776 (9.63)     1,200.0019 (5.41)     100.0008 (5.00)       387;2129      771.2678 (0.20)      45872           1
test_ul_bench[mercantile-(1, 0, 1)]            999.9931 (4.89)     166,500.0018 (23.25)    1,288.3700 (4.89)       712.6090 (5.71)     1,299.9953 (5.86)     100.0008 (5.00)      2119;3450      776.1746 (0.20)     147059           1
test_ul_bench[mercantile-(1, 1, 1)]          1,000.0003 (4.89)     102,799.9970 (14.36)    1,253.0401 (4.75)       570.2565 (4.57)     1,200.0019 (5.41)     100.0008 (5.00)      2957;3697      798.0590 (0.21)     144928           1
test_ul_bench[mercantile-(1, 40, 7)]         1,000.0003 (4.89)      89,599.9983 (12.51)    1,263.1955 (4.79)       586.9464 (4.70)     1,200.0019 (5.41)     100.0008 (5.00)      1775;2965      791.6431 (0.21)     166667           1
test_ul_bench[mercantile-(486, 332, 10)]     1,099.9938 (5.38)      90,200.0029 (12.60)    1,327.0801 (5.03)       536.7494 (4.30)     1,299.9953 (5.86)     100.0008 (5.00)      6813;7956      753.5340 (0.20)     135136           1
test_ul_bench[mercantile-(486, 332, 20)]     1,099.9938 (5.38)     107,300.0021 (14.98)    1,264.2361 (4.79)       594.6154 (4.77)     1,200.0019 (5.41)     100.0008 (5.00)      1522;2265      790.9915 (0.21)     123457           1
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
```

## TODO:

- [x] benchmark against mercantile
- [x] Split library into `utiles` (rust lib) and `utiles-python` (python/pip package)?
- [x] Re-write cli in rust with clap?
- **Maybe:**
  - [] Mbtiles support for the python lib??
  - [] Reading/writing mvt files?
