from __future__ import annotations

from pathlib import Path
from typing import Any, Callable

import mercantile
import pytest
from pmtiles.tile import tileid_to_zxy, zxy_to_tileid
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
    tile: tuple[int, int, int],
    func: Callable[[tuple[int, int, int]], str],
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
    tile: tuple[int, int, int],
    func: Callable[[tuple[int, int, int]], tuple[float, float]],
    benchmark: BenchmarkFixture,
) -> None:
    benchmark(func, *tile)


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
    assert list(mercantile._coords(obj)) == [(1, 2)]


def utiles_coords(obj: Any) -> None:
    assert list(utiles._coords(obj)) == [(1, 2)]


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
    func: Callable[[Any], tuple[float, float] | list[tuple[float, float]]],
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


# ===================================================================
# PMTILES ~ PMTILES ~ PMTILES ~ PMTILES ~ PMTILES ~ PMTILES ~ PMTILES
# ===================================================================
def _ut_xyz2pmtileid(x: int, y: int, z: int) -> int:
    return utiles.pmtileid(x, y, z)


def _pm_xyz2pmtileid(x: int, y: int, z: int) -> int:
    return zxy_to_tileid(z, x, y)


def _ut_pmtileid2xyz(tileid: int) -> tuple[int, int, int]:
    return utiles.from_pmtileid(tileid)


def _pm_pmtileid2xyz(tileid: int) -> tuple[int, int, int]:
    return tileid_to_zxy(tileid)


def test_xyz2pmtileid_eq():
    for tile in TEST_TILES:
        x, y, z = tile
        pmtileid_from_pmtiles = zxy_to_tileid(
            z,
            x,
            y,
        )
        pmtileid_from_utiles = utiles.pmtileid(x, y, z)
        assert pmtileid_from_pmtiles == pmtileid_from_utiles

        # round trip
        (x, y, z) = _ut_pmtileid2xyz(pmtileid_from_utiles)
        zxy_from_pmtiles = tileid_to_zxy(pmtileid_from_pmtiles)
        assert (z, x, y) == zxy_from_pmtiles


@pytest.mark.benchmark(
    group="xyz2pmtile",
)
@pytest.mark.parametrize(
    "tile",
    [pytest.param(t, id=str(t)) for t in TEST_TILES],
)
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(_pm_xyz2pmtileid, id="pmtiles"),
        pytest.param(_ut_xyz2pmtileid, id="utiles"),
    ],
)
def test_xyz2pmtileid(
    tile: tuple[int, int, int],
    func: Callable[[int, int, int], list[str]],
    benchmark: BenchmarkFixture,
) -> None:
    """Get feature of tile"""
    benchmark(func, *tile)


@pytest.mark.benchmark(
    group="pmtile2xyz",
)
@pytest.mark.parametrize(
    "tile",
    [pytest.param(t, id=str(t)) for t in TEST_TILES],
)
@pytest.mark.parametrize(
    "func",
    [
        pytest.param(tileid_to_zxy, id="pmtiles"),
        pytest.param(utiles.pmtileid2xyz, id="utiles"),
    ],
)
def test_pmtileid2xyz(
    tile: tuple[int, int, int],
    func: Callable[[int, int, int], list[str]],
    benchmark: BenchmarkFixture,
) -> None:
    """Get feature of tile"""
    pmid = utiles.pmtileid(*tile)
    benchmark(func, pmid)
