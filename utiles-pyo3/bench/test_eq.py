from typing import Any

from pytest_benchmark.fixture import BenchmarkFixture

import utiles


def test_tile_equality() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    assert tile_obj == t  # noqa: S101


def _equal(a: Any, b: Any) -> bool:
    return bool(a == b)


def test_tile_equality_tuple2tuple(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    t2 = (1, 2, 3)
    benchmark(_equal, t, t2)


def test_tile_equality_tuple2tile(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    benchmark(_equal, t, tile_obj)


def test_tile_equality_tile2tuple(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    benchmark(_equal, tile_obj, t)


def test_tile_equality_tile2tile(benchmark: BenchmarkFixture) -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)

    benchmark(_equal, tile_obj, tile_obj)
