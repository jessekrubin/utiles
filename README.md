# utiles

utiles = utils + tiles

## Installation

```bash
# python (python lib + rust-cli)
pip install -U utiles 
# rust-cli
cargo install utiles
# rust-libs
cargo add utiles-core utiles
```

## python

A mostly drop-in replacement for [mercantile](https://github.com/mapbox/mercantile) written w/ rust, plus several other util(e)ities

[py-utiles](https://github.com/jessekrubin/utiles/tree/main/utiles-pyo3)

## About

`utiles` started off as a python port of mapbox's web-mercator utils
python-library [mercantile](https://github.com/mapbox/mercantile) written
in rust. It has since been expanded into a slim rust crate (`utiles-core`)
a less slim crate with a lib/cli (`utiles`), and the python wrapper package.

For more details on the python package see: [./utiles-pyo3](https://github.com/jessekrubin/utiles/tree/main/utiles-pyo3)

### Why?

I use mercantile regularly and wished it were a bit more ergonomic, had type annotations, and was faster, but overall
it's a great library.

This was an excuse to learn some more rust as well as pyo3.

**Do I/you REALLY need a rust-port of mercantile?**

I don't know, decide for yourself. `utiles` is certainly faster than `mercantile` for some things (see benchmarks below)

**Is it really a drop in replacement for mercantile?**

Not quite, but it's close. utiles doesn't throw the same exceptions as mercantile, instead it throws `ValueError`'s and
`TypeError`'s.

There might be other differences, but I have been using it instead of mercantile for a bit now and it works pretty decent, tho I am open to suggestions!

---

# dev

## Contributing

- Please do! Would love some feedback!
- Be kind!
- I will happily accept PRs, and add you to the currently (5/26/2023) non-existent contributors list.

## TODO:

- [x] benchmark against mercantile
- **Maybe:**
  - [x] Split library into `utiles` (rust lib) and `utiles-python` (python/pip package)?
  - [] Mbtiles support??
  - [] Reading/writing mvt files?
  - [] Re-write cli in rust with clap?

---

## MISC

<details>
<summary>zoom info</summary>

| zoom |                    ntiles |                     total |  rowcol_range |    max_rowcol |
| ---: | ------------------------: | ------------------------: | ------------: | ------------: |
|    0 |                         1 |                         1 |             0 |             1 |
|    1 |                         4 |                         5 |             1 |             2 |
|    2 |                        16 |                        21 |             3 |             4 |
|    3 |                        64 |                        85 |             7 |             8 |
|    4 |                       256 |                       341 |            15 |            16 |
|    5 |                     1_024 |                     1_365 |            31 |            32 |
|    6 |                     4_096 |                     5_461 |            63 |            64 |
|    7 |                    16_384 |                    21_845 |           127 |           128 |
|    8 |                    65_536 |                    87_381 |           255 |           256 |
|    9 |                   262_144 |                   349_525 |           511 |           512 |
|   10 |                 1_048_576 |                 1_398_101 |         1_023 |         1_024 |
|   11 |                 4_194_304 |                 5_592_405 |         2_047 |         2_048 |
|   12 |                16_777_216 |                22_369_621 |         4_095 |         4_096 |
|   13 |                67_108_864 |                89_478_485 |         8_191 |         8_192 |
|   14 |               268_435_456 |               357_913_941 |        16_383 |        16_384 |
|   15 |             1_073_741_824 |             1_431_655_765 |        32_767 |        32_768 |
|   16 |             4_294_967_296 |             5_726_623_061 |        65_535 |        65_536 |
|   17 |            17_179_869_184 |            22_906_492_245 |       131_071 |       131_072 |
|   18 |            68_719_476_736 |            91_625_968_981 |       262_143 |       262_144 |
|   19 |           274_877_906_944 |           366_503_875_925 |       524_287 |       524_288 |
|   20 |         1_099_511_627_776 |         1_466_015_503_701 |     1_048_575 |     1_048_576 |
|   21 |         4_398_046_511_104 |         5_864_062_014_805 |     2_097_151 |     2_097_152 |
|   22 |        17_592_186_044_416 |        23_456_248_059_221 |     4_194_303 |     4_194_304 |
|   23 |        70_368_744_177_664 |        93_824_992_236_885 |     8_388_607 |     8_388_608 |
|   24 |       281_474_976_710_656 |       375_299_968_947_541 |    16_777_215 |    16_777_216 |
|   25 |     1_125_899_906_842_624 |     1_501_199_875_790_165 |    33_554_431 |    33_554_432 |
|   26 |     4_503_599_627_370_496 |     6_004_799_503_160_661 |    67_108_863 |    67_108_864 |
|   27 |    18_014_398_509_481_984 |    24_019_198_012_642_645 |   134_217_727 |   134_217_728 |
|   28 |    72_057_594_037_927_936 |    96_076_792_050_570_581 |   268_435_455 |   268_435_456 |
|   29 |   288_230_376_151_711_744 |   384_307_168_202_282_325 |   536_870_911 |   536_870_912 |
|   30 | 1_152_921_504_606_846_976 | 1_537_228_672_809_129_301 | 1_073_741_823 | 1_073_741_824 |
|   31 | 4_611_686_018_427_387_904 | 6_148_914_691_236_517_205 | 2_147_483_647 | 2_147_483_648 |

</details>

Zoom levels

```
    zoom               ntiles                total  rowcol_range  max_rowcol
0      0                    1                    1             0           1
1      1                    4                    5             1           2
2      2                   16                   21             3           4
3      3                   64                   85             7           8
4      4                  256                  341            15          16
5      5                 1024                 1365            31          32
6      6                 4096                 5461            63          64
7      7                16384                21845           127         128
8      8                65536                87381           255         256
9      9               262144               349525           511         512
10    10              1048576              1398101          1023        1024
11    11              4194304              5592405          2047        2048
12    12             16777216             22369621          4095        4096
13    13             67108864             89478485          8191        8192
14    14            268435456            357913941         16383       16384
15    15           1073741824           1431655765         32767       32768
16    16           4294967296           5726623061         65535       65536
17    17          17179869184          22906492245        131071      131072
18    18          68719476736          91625968981        262143      262144
19    19         274877906944         366503875925        524287      524288
20    20        1099511627776        1466015503701       1048575     1048576
21    21        4398046511104        5864062014805       2097151     2097152
22    22       17592186044416       23456248059221       4194303     4194304
23    23       70368744177664       93824992236885       8388607     8388608
24    24      281474976710656      375299968947541      16777215    16777216
25    25     1125899906842624     1501199875790165      33554431    33554432
26    26     4503599627370496     6004799503160661      67108863    67108864
27    27    18014398509481984    24019198012642645     134217727   134217728
28    28    72057594037927936    96076792050570581     268435455   268435456
29    29   288230376151711744   384307168202282325     536870911   536870912
30    30  1152921504606846976  1537228672809129301    1073741823  1073741824
31    31  4611686018427387904  6148914691236517205    2147483647  2147483648
```

<details>
<summary>json</summary>

```json
[
  {
    "max_rowcol": 1,
    "ntiles": 1,
    "rowcol_range": 0,
    "total": 1,
    "zoom": 0
  },
  {
    "max_rowcol": 2,
    "ntiles": 4,
    "rowcol_range": 1,
    "total": 5,
    "zoom": 1
  },
  {
    "max_rowcol": 4,
    "ntiles": 16,
    "rowcol_range": 3,
    "total": 21,
    "zoom": 2
  },
  {
    "max_rowcol": 8,
    "ntiles": 64,
    "rowcol_range": 7,
    "total": 85,
    "zoom": 3
  },
  {
    "max_rowcol": 16,
    "ntiles": 256,
    "rowcol_range": 15,
    "total": 341,
    "zoom": 4
  },
  {
    "max_rowcol": 32,
    "ntiles": 1024,
    "rowcol_range": 31,
    "total": 1365,
    "zoom": 5
  },
  {
    "max_rowcol": 64,
    "ntiles": 4096,
    "rowcol_range": 63,
    "total": 5461,
    "zoom": 6
  },
  {
    "max_rowcol": 128,
    "ntiles": 16384,
    "rowcol_range": 127,
    "total": 21845,
    "zoom": 7
  },
  {
    "max_rowcol": 256,
    "ntiles": 65536,
    "rowcol_range": 255,
    "total": 87381,
    "zoom": 8
  },
  {
    "max_rowcol": 512,
    "ntiles": 262144,
    "rowcol_range": 511,
    "total": 349525,
    "zoom": 9
  },
  {
    "max_rowcol": 1024,
    "ntiles": 1048576,
    "rowcol_range": 1023,
    "total": 1398101,
    "zoom": 10
  },
  {
    "max_rowcol": 2048,
    "ntiles": 4194304,
    "rowcol_range": 2047,
    "total": 5592405,
    "zoom": 11
  },
  {
    "max_rowcol": 4096,
    "ntiles": 16777216,
    "rowcol_range": 4095,
    "total": 22369621,
    "zoom": 12
  },
  {
    "max_rowcol": 8192,
    "ntiles": 67108864,
    "rowcol_range": 8191,
    "total": 89478485,
    "zoom": 13
  },
  {
    "max_rowcol": 16384,
    "ntiles": 268435456,
    "rowcol_range": 16383,
    "total": 357913941,
    "zoom": 14
  },
  {
    "max_rowcol": 32768,
    "ntiles": 1073741824,
    "rowcol_range": 32767,
    "total": 1431655765,
    "zoom": 15
  },
  {
    "max_rowcol": 65536,
    "ntiles": 4294967296,
    "rowcol_range": 65535,
    "total": 5726623061,
    "zoom": 16
  },
  {
    "max_rowcol": 131072,
    "ntiles": 17179869184,
    "rowcol_range": 131071,
    "total": 22906492245,
    "zoom": 17
  },
  {
    "max_rowcol": 262144,
    "ntiles": 68719476736,
    "rowcol_range": 262143,
    "total": 91625968981,
    "zoom": 18
  },
  {
    "max_rowcol": 524288,
    "ntiles": 274877906944,
    "rowcol_range": 524287,
    "total": 366503875925,
    "zoom": 19
  },
  {
    "max_rowcol": 1048576,
    "ntiles": 1099511627776,
    "rowcol_range": 1048575,
    "total": 1466015503701,
    "zoom": 20
  },
  {
    "max_rowcol": 2097152,
    "ntiles": 4398046511104,
    "rowcol_range": 2097151,
    "total": 5864062014805,
    "zoom": 21
  },
  {
    "max_rowcol": 4194304,
    "ntiles": 17592186044416,
    "rowcol_range": 4194303,
    "total": 23456248059221,
    "zoom": 22
  },
  {
    "max_rowcol": 8388608,
    "ntiles": 70368744177664,
    "rowcol_range": 8388607,
    "total": 93824992236885,
    "zoom": 23
  },
  {
    "max_rowcol": 16777216,
    "ntiles": 281474976710656,
    "rowcol_range": 16777215,
    "total": 375299968947541,
    "zoom": 24
  },
  {
    "max_rowcol": 33554432,
    "ntiles": 1125899906842624,
    "rowcol_range": 33554431,
    "total": 1501199875790165,
    "zoom": 25
  },
  {
    "max_rowcol": 67108864,
    "ntiles": 4503599627370496,
    "rowcol_range": 67108863,
    "total": 6004799503160661,
    "zoom": 26
  },
  {
    "max_rowcol": 134217728,
    "ntiles": 18014398509481984,
    "rowcol_range": 134217727,
    "total": 24019198012642645,
    "zoom": 27
  },
  {
    "max_rowcol": 268435456,
    "ntiles": 72057594037927936,
    "rowcol_range": 268435455,
    "total": 96076792050570581,
    "zoom": 28
  },
  {
    "max_rowcol": 536870912,
    "ntiles": 288230376151711744,
    "rowcol_range": 536870911,
    "total": 384307168202282325,
    "zoom": 29
  },
  {
    "max_rowcol": 1073741824,
    "ntiles": 1152921504606846976,
    "rowcol_range": 1073741823,
    "total": 1537228672809129301,
    "zoom": 30
  },
  {
    "max_rowcol": 2147483648,
    "ntiles": 4611686018427387904,
    "rowcol_range": 2147483647,
    "total": 6148914691236517205,
    "zoom": 31
  }
]
```

</details>
