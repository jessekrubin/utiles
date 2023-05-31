from pathlib import Path
from typing import Any, Callable, List, Tuple, Union

import mercantile
import pytest
from pytest_benchmark.fixture import BenchmarkFixture

import utiles

# mark all as benchmarks
pytestmark = [pytest.mark.benchmark(group="utiles"), pytest.mark.bench]

PWD = Path(__file__).parent
REPO_ROOT = PWD.parent

TEST_TILES = (
    (0, 0, 0),
    (1, 0, 1),
    (1, 1, 1),
    (1, 40, 7),
    (486, 332, 10),
    #     REALLY HIGH ZOOM
    (486, 332, 20),
)

tile_pytest_params = pytest.mark.parametrize(
    "tile",
    [pytest.param(t, id=str(t)) for t in TEST_TILES],
)


# @pytest.mark.parametrize(
#     "tile",
#     [
#         pytest.param(t, id=str(t)) for t in TEST_TILES
#     ],
# )
@tile_pytest_params
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(mercantile.quadkey, id="mercantile"),
        pytest.param(utiles.quadkey, id="utiles"),
    ],
)
@pytest.mark.benchmark(
    group="quadkey",
)
def test_quadkey_bench(
    tile: Tuple[int, int, int],
    func: Callable[[Tuple[int, int, int]], str],
    benchmark: BenchmarkFixture,
) -> None:
    benchmark(func, *tile)


@pytest.mark.parametrize(
    "tile",
    [pytest.param(t, id=str(t)) for t in TEST_TILES],
)
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(mercantile.ul, id="mercantile"),
        pytest.param(utiles.ul, id="utiles"),
    ],
)
@pytest.mark.benchmark(
    group="ul",
)
def test_ul_bench(
    tile: Tuple[int, int, int],
    func: Callable[[Tuple[int, int, int]], Tuple[float, float]],
    benchmark: BenchmarkFixture,
) -> None:
    benchmark(func, *tile)


# @pytest.mark.parametrize(
#     "tile",
#     [pytest.param(t, id=str(t)) for t in TEST_TILES],
# )
# @pytest.mark.parametrize(
#     "func",
#     [
#         pytest.param(mercantile.ul, id="mercantile"),
#         pytest.param(utiles.ul, id="utiles"),
#     ],
# )
# def test_ul_bench(
#     tile: Tuple[int, int, int],
#     func: Callable[[Tuple[int, int, int]], Tuple[float, float]],
#     benchmark: BenchmarkFixture,
# ) -> None:
#     benchmark(func, *tile)


def mercantile_tiles_gen() -> None:
    for _tile in mercantile.tiles(-180, -85, 180, 85, 6):
        pass


def utiles_tiles_gen() -> None:
    for _tile in utiles.tiles(-180, -85, 180, 85, 6):
        pass


@pytest.mark.parametrize(
    "func",
    [
        pytest.param(mercantile_tiles_gen, id="mercantile"),
        pytest.param(utiles_tiles_gen, id="utiles"),
    ],
)
@pytest.mark.benchmark(
    group="tiles",
)
def test_tiles_gen_bench(func: Callable[[], None], benchmark: BenchmarkFixture) -> None:
    benchmark(func)


# ========================================================================
# COORDS BENCH ~ COORDS BENCH ~ COORDS BENCH ~ COORDS BENCH ~ COORDS BENCH
# ========================================================================
def mercantile_coords(obj: Any) -> None:
    assert list(mercantile._coords(obj)) == [(1, 2)]  # noqa: S101


def utiles_coords(obj: Any) -> None:
    assert list(utiles._coords(obj)) == [(1, 2)]  # noqa: S101


@pytest.mark.benchmark(
    group="coords",
)
@pytest.mark.parametrize(
    "obj",
    [
        {"features": [{"geometry": {"coordinates": (1, 2)}}]},
        {"geometry": {"coordinates": (1, 2)}},
        {"coordinates": (1, 2)},
        {"coordinates": [(1, 2)]},
        (1, 2),
        [(1, 2)],
    ],
)
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(mercantile_coords, id="mercantile"),
        pytest.param(utiles_coords, id="utiles"),
    ],
)
def test_coords(
    func: Callable[[Any], Union[Tuple[float, float], List[Tuple[float, float]]]],
    obj: Any,
    benchmark: BenchmarkFixture,
) -> None:
    """Get coordinates of mock geojson objects"""
    benchmark(func, obj)


@pytest.mark.benchmark(
    group="feature",
)
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(mercantile.feature, id="mercantile"),
        pytest.param(utiles.feature, id="utiles"),
    ],
)
def test_feature(
    func: Callable[[mercantile.Tile], dict], benchmark: BenchmarkFixture
) -> None:
    """Get feature of tile"""
    benchmark(func, mercantile.Tile(1, 2, 3))
