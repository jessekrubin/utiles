"""utiles"""
from __future__ import annotations

import math
from typing import List, Sequence, Tuple, Union

from utiles.libutiles import (
    TILETYPE_GIF,
    TILETYPE_JPG,
    TILETYPE_PBF,
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
    from_pmtileid,
    from_tuple,
    geojson_bounds,
    lnglat,
    minmax,
    neighbors,
    parent,
    parse_tile_arg,
    parse_tiles,
    pmtileid,
    quadkey,
    quadkey2xyz,
    quadkey_to_tile,
    simplify,
    tile,
    tiles,
    tiles_list,
    tiletype,
    tiletype2headers,
    tiletype_str,
    truncate_lnglat,
    ul,
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
    "TILETYPE_PBF",
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
    "from_pmtileid",
    "from_tuple",
    "geojson_bounds",
    "lnglat",
    "minmax",
    "neighbors",
    "parent",
    "parse_tile_arg",
    "parse_tiles",
    "pmtileid",
    "quadkey",
    "quadkey2xyz",
    "quadkey_to_tile",
    "simplify",
    "tile",
    "tiles",
    "tiles_list",
    "tiletype",
    "tiletype2headers",
    "tiletype_str",
    "truncate_lnglat",
    "ul",
    "xy",
    "xy_bounds",
    "xyz",
    "xyz2quadkey",
)


def optzoom(
    geo_transform: Union[
        Tuple[float, float, float, float, float, float],
        List[float],
    ],
    pixel_size: int = 256,
) -> int:
    """Return the optimal zoom level for a given geo_transform

    Args:
        geo_transform (Sequence[float]): Geo transform array

    Returns:
        int: Optimal zoom level

    Example:
        >>> gt = [-77.000138, 0.000278, 0.0, 26.0001389, 0.0, -0.000278]
        >>> optzoom(gt)
        12

    """
    degrees_per_pixel = geo_transform[1]
    equator = 2 * math.pi * 6378137  # 2 * pi * radius of earth in meters
    resolution = degrees_per_pixel * (equator / 360)
    zoom_level = math.log((equator / pixel_size) / resolution, 2)
    return min(math.floor(zoom_level), 20)


def tiletile_str(n: int) -> str:
    if n == TILETYPE_PNG:
        return "png"
    elif n == TILETYPE_JPG:
        return "jpg"
    elif n == TILETYPE_GIF:
        return "gif"
    elif n == TILETYPE_WEBP:
        return "webp"
    elif n == TILETYPE_PBF:
        return "pbf"
    else:
        return "unknown"
