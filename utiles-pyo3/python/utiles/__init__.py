"""utiles

Rust + python spherical mercator coordinate and tile util(e)ities

Example:
    >>> import utiles as ut
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

"""

from __future__ import annotations

from typing import Sequence, Tuple, Union

from utiles._utiles import (
    TILETYPE_GIF,
    TILETYPE_JPG,
    TILETYPE_JSON,
    TILETYPE_PBF,
    TILETYPE_PBFGZ,
    TILETYPE_PNG,
    TILETYPE_UNKNOWN,
    TILETYPE_WEBP,
    Bbox,
    LngLat,
    LngLatBbox,
    Tile,
    __build_profile__,
    __version_lib__,
    _coords,
    _parse_tile_arg,
    _xy,
    bounding_tile,
    bounds,
    children,
    coords,
    feature,
    fmt_nbytes,
    from_pmtileid,
    from_tuple,
    geojson_bounds,
    geotransform2optzoom,
    lnglat,
    minmax,
    neighbors,
    parent,
    parse_tile_arg,
    parse_tiles,
    pmtileid,
    pmtileid2xyz,
    qk2xyz,
    quadkey,
    quadkey2xyz,
    quadkey_to_tile,
    simplify,
    tile,
    tiles,
    tiles_count,
    tiles_list,
    tiletype,
    tiletype2headers,
    tiletype_str,
    truncate_lnglat,
    ul,
    ut_cli,
    xy,
    xy_bounds,
    xyz,
    xyz2quadkey,
)

TileLike = Union[Tile, Tuple[int, int, int]]
TileArg = Union[TileLike, Sequence[TileLike]]

__version__ = __version_lib__
__all__ = (
    "Bbox",
    "LngLat",
    "LngLatBbox",
    "TILETYPE_GIF",
    "TILETYPE_JPG",
    "TILETYPE_JSON",
    "TILETYPE_PBF",
    "TILETYPE_PBFGZ",
    "TILETYPE_PNG",
    "TILETYPE_UNKNOWN",
    "TILETYPE_WEBP",
    "Tile",
    "__build_profile__",
    "__version__",
    "__version_lib__",
    "_coords",
    "_parse_tile_arg",
    "_xy",
    "bounding_tile",
    "bounds",
    "children",
    "coords",
    "feature",
    "fmt_nbytes",
    "from_pmtileid",
    "from_tuple",
    "geojson_bounds",
    "geotransform2optzoom",
    "lnglat",
    "minmax",
    "neighbors",
    "parent",
    "parse_tile_arg",
    "parse_tiles",
    "pmtileid",
    "pmtileid2xyz",
    "qk2xyz",
    "quadkey",
    "quadkey2xyz",
    "quadkey_to_tile",
    "simplify",
    "tile",
    "tiles",
    "tiles_count",
    "tiles_list",
    "tiletype",
    "tiletype2headers",
    "tiletype_str",
    "truncate_lnglat",
    "ul",
    "ut_cli",
    "xy",
    "xy_bounds",
    "xyz",
    "xyz2quadkey",
)
