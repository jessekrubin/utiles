from __future__ import annotations

from pathlib import Path
from typing import Any, List, Tuple, Union

import pytest
from pytest_benchmark.fixture import BenchmarkFixture
from typing_extensions import TypedDict

import utiles
from utiles import Tile

PWD = Path(__file__).parent
REPO_ROOT = PWD.parent


def test_version() -> None:
    assert utiles.__version__ is not None
    import tomli

    Path("Cargo.toml").read_text()
    cargo_version = tomli.loads(Path("Cargo.toml").read_text())["package"]["version"]
    assert utiles.__version__ == cargo_version
    pyproject_version = tomli.loads(Path("pyproject.toml").read_text())["project"][
        "version"
    ]
    assert utiles.__version__ == pyproject_version


class TileDict(TypedDict):
    x: int
    y: int
    z: int


@pytest.mark.parametrize(
    "tile,quadkey",
    [
        ((0, 0, 0), ""),
        ((1, 0, 1), "1"),
        ((1, 1, 1), "3"),
        ((486, 332, 10), "0313102310"),
    ],
)
def test_quadkey(tile: Tuple[int, int, int], quadkey: str) -> None:
    # mtile = tile_dict_to_mercantile_tile(tile_dict)
    # expected = mercantile.quadkey(mtile)

    utiles_qk = utiles.xyz2quadkey(*tile)
    assert utiles_qk == quadkey, f"utiles: {utiles_qk} ~ mercantile: {quadkey}"
    assert utiles.xyz2quadkey(0, 0, 0) == ""  # noqa: PLC1901
    assert utiles.xyz2quadkey(1, 0, 1) == "1"
    assert utiles.quadkey(utiles.Tile(1, 0, 1)) == "1"
    assert utiles.xyz2quadkey(1, 1, 1) == "3"
    assert utiles.xyz2quadkey(486, 332, 10) == "0313102310"


tile123 = utiles.Tile(1, 2, 3)


def test_tile_obj() -> None:
    t = utiles.Tile(1, 2, 3)
    assert str(t) == "Tile(x=1, y=2, z=3)"

    assert t.x == 1
    assert t.y == 2
    assert t.z == 3

    assert t[0] == 1
    assert t[1] == 2
    assert t[2] == 3

    assert t[-1] == 3
    assert t[-2] == 2
    assert t[-3] == 1

    flipped = t.flipy()
    assert flipped.y == (2**3) - 1 - 2
    children = t.children()
    assert len(children) == 4
    expected_children = [
        Tile(x=2, y=4, z=4),
        Tile(x=3, y=4, z=4),
        Tile(x=3, y=5, z=4),
        Tile(x=2, y=5, z=4),
    ]
    assert children == expected_children
    parent = t.parent()

    assert parent == Tile(x=0, y=1, z=2)


def test_tile_tuple() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    as_tuple_again = tuple(tile_obj)
    assert as_tuple_again == t
    assert tuple(tile_obj) == (1, 2, 3)
    assert tile_obj.tuple() == (1, 2, 3)


def test_tile_parse_if_already_utiles_tile() -> None:
    t = utiles.Tile(1, 2, 3)
    tile_obj = utiles.parse_tile_arg(t)
    assert tile_obj == t
    assert isinstance(tile_obj, utiles.Tile)


def test_parse_tiles() -> None:
    tile_obj = utiles.Tile(7, 8, 9)
    t: List[Union[Tuple[int, int, int], utiles.Tile]] = [(1, 2, 3), (4, 5, 6), tile_obj]
    tiles_list = utiles.parse_tiles(t)
    assert tiles_list == [
        utiles.Tile(1, 2, 3),
        utiles.Tile(4, 5, 6),
        utiles.Tile(7, 8, 9),
    ]
    assert all(isinstance(t, utiles.Tile) for t in tiles_list)


# def test_parse_tiles_spread() -> None:
#     tile_obj = utiles.Tile(
#         7, 8, 9
#     )
#     t: List[Union[
#         Tuple[int, int, int], utiles.Tile
#     ]] = [(1, 2, 3), (4, 5, 6),  tile_obj]
#     tiles_list = utiles.parse_tiles(*t)
#     assert tiles_list == [
#         utiles.Tile(1, 2, 3),
#         utiles.Tile(4, 5, 6),
#         utiles.Tile(7, 8, 9),
#     ]
#     assert all(isinstance(t, utiles.Tile) for t in tiles_list)


def test_tile_equality() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    assert tile_obj == t


def test_tile_asdict() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    d = tile_obj.asdict()
    assert d == {"x": 1, "y": 2, "z": 3}


def test_lnglat_equality() -> None:
    t = (50, 50)
    lnglat = utiles.LngLat(*t)
    assert lnglat == t


def test_bbox_equality() -> None:
    t = (50, 50, 60, 60)
    bbox = utiles.Bbox(*t)
    assert bbox == t


def test_lnglat_bbox_equality() -> None:
    t = (50, 50, 60, 60)
    bbox = utiles.LngLatBbox(*t)
    assert bbox == t


def _equal(a: Any, b: Any) -> bool:
    return bool(a == b)


def test_tile_equality_tuple2tuple(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    t2 = (1, 2, 3)
    # tile_obj = utiles.from_tuple(t)
    benchmark(_equal, t, t2)


def test_tile_equality_tile2tuple(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    benchmark(_equal, tile_obj, t)


def test_tile_equality_tile2tile(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)

    benchmark(_equal, tile_obj, tile_obj)


def test_tile_slice() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    sliced = tile_obj[:2]
    assert isinstance(sliced, tuple)
    assert sliced == (1, 2)

    sliced_with_step = tile_obj[::2]
    assert isinstance(sliced_with_step, tuple)
    assert sliced_with_step == (1, 3)


def test_tile_iter() -> None:
    for ix, el in enumerate(tile123):
        assert ix == el - 1


def test_tile_spread() -> None:
    t = utiles.Tile(x=1, y=2, z=3)
    x, y, z = t
    assert x == 1
    assert y == 2
    assert z == 3


def test_tile_invert() -> None:
    t = utiles.Tile(x=1, y=2, z=3)
    assert str(t) == "Tile(x=1, y=2, z=3)"

    assert t.x == 1
    assert t.y == 2
    assert t.z == 3

    flipped = t.flipy()
    assert flipped.y == (2**3) - 1 - 2

    assert ~t == flipped


def test_tile_kwargs() -> None:
    t = utiles.Tile(x=1, y=2, z=3)
    assert str(t) == "Tile(x=1, y=2, z=3)"

    assert t.x == 1
    assert t.y == 2
    assert t.z == 3


def test_parse_tile_arg() -> None:
    toop = (1, 2, 3)
    tile_obj = utiles.Tile(1, 2, 3)
    assert utiles.parse_tile_arg(toop) == tile_obj
    assert utiles.parse_tile_arg(1, 2, 3) == tile_obj
    assert utiles.parse_tile_arg(utiles.Tile(1, 2, 3)) == tile_obj


def test_tile_feature() -> None:
    t = utiles.Tile(1, 2, 3)
    f = utiles.feature(t, props={"x": 1, "y": 2, "z": 3})
    expected = {
        "bbox": [-135.0, 40.97989806962013, -90.0, 66.51326044311186],
        "type": "Feature",
        "geometry": {
            "type": "Polygon",
            "coordinates": [
                [
                    [-135.0, 40.97989806962013],
                    [-135.0, 66.51326044311186],
                    [-90.0, 66.51326044311186],
                    [-90.0, 40.97989806962013],
                    [-135.0, 40.97989806962013],
                ]
            ],
        },
        "id": "(1, 2, 3)",
        "properties": {"title": "XYZ tile (1, 2, 3)", "z": 3, "y": 2, "x": 1},
    }

    assert f == expected
    assert t.feature(props={"x": 1, "y": 2, "z": 3}) == expected
