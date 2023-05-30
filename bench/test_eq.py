import utiles
from pytest_benchmark.fixture import BenchmarkFixture
from typing import Any

def test_tile_equality() -> None:
    t = (1, 2, 3)
    tile_obj = utiles.from_tuple(t)
    assert tile_obj == t


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
