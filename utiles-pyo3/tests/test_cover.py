from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from pprint import pprint

import pytest

import utiles as ut

PWD = Path(__file__).parent


@lru_cache(maxsize=1)
def _repo_root() -> Path:
    _root = PWD
    for _i in range(5):
        _root = _root.parent
        if (_root / ".github").is_dir():
            return _root
    ex = RuntimeError("Could not find repo root")
    raise ex


TEST_COVER_LIMITS_MAP = {
    "road.geojson": {"minzoom": 1, "maxzoom": 12},
    "world.geojson": {"minzoom": 1, "maxzoom": 6},
    "point.geojson": {"minzoom": 1, "maxzoom": 15},
    "line.geojson": {"minzoom": 1, "maxzoom": 12},
    "edgeline.geojson": {"minzoom": 15, "maxzoom": 15},
    "polygon.geojson": {"minzoom": 1, "maxzoom": 15},
    "multipoint.geojson": {"minzoom": 1, "maxzoom": 12},
    "multiline.geojson": {"minzoom": 1, "maxzoom": 8},
    "uk.geojson": {"minzoom": 7, "maxzoom": 9},
    "building.geojson": {"minzoom": 18, "maxzoom": 18},
    "donut.geojson": {"minzoom": 16, "maxzoom": 16},
    "russia.geojson": {"minzoom": 6, "maxzoom": 6},
    "degenring.geojson": {"minzoom": 11, "maxzoom": 15},
    "invalid_polygon.geojson": {"minzoom": 1, "maxzoom": 12},
    "highzoom.geojson": {"minzoom": 23, "maxzoom": 23},
    "small_poly.geojson": {"minzoom": 10, "maxzoom": 10},
    "spiked.geojson": {"minzoom": 10, "maxzoom": 10},
    "blocky.geojson": {"minzoom": 6, "maxzoom": 6},
    "pyramid.geojson": {"minzoom": 10, "maxzoom": 10},
    "tetris.geojson": {"minzoom": 10, "maxzoom": 10},
    "zero.geojson": {"minzoom": 10, "maxzoom": 10},
}

TILE_COVER_FILEPATHS = _repo_root() / "test-data" / "tile-cover"
TEST_GEOJSON_FILEPATHS = [
    e for e in TILE_COVER_FILEPATHS.rglob("*.geojson") if "world" not in str(e)
]


def zoom_limits(filepath: Path | str) -> tuple[int, int]:
    filename = Path(filepath).name
    if filename in TEST_COVER_LIMITS_MAP:
        try:
            lims = TEST_COVER_LIMITS_MAP[filename]
            return lims["minzoom"], lims["maxzoom"]
        except KeyError:
            pass
    return (1, 6)


def _test_geojson_cover(geojson_filepath: str) -> None:
    with open(geojson_filepath, "r") as f:
        data = f.read()
    expected_textiles_filepath = str(geojson_filepath).replace(
        ".geojson", ".tiles.jsonl"
    )
    with open(expected_textiles_filepath, "r") as f:
        textiles = f.read()

    minzoom, maxzoom = zoom_limits(geojson_filepath)
    expected_tiles = set(ut.parse_textiles(textiles))
    coverage = set(ut.geojson2tiles(data, maxzoom, minzoom))

    if set(expected_tiles) != set(coverage):
        not_in_expected_tiles = expected_tiles.difference(coverage)
        not_in_coverage = coverage.difference(expected_tiles)
        print("============")
        print(geojson_filepath)
        pprint(
            {
                "not_in_expected_tiles": not_in_expected_tiles,
                "not_in_coverage": not_in_coverage,
                # 'common_tiles': common_tiles,
            }
        )
    assert set(expected_tiles) == set(coverage)


PROBLEM_CHILDREN = {
    "degenring.geojson",
    "donut.geojson",
    "highzoom.geojson",
    "multipoint.geojson",
    "spiked.geojson",
    "uk.geojson",
}


@pytest.mark.parametrize(
    "filepath",
    [
        pytest.param(str(e), id=str(e))
        for e in TEST_GEOJSON_FILEPATHS
        if not any(str(e).endswith(p) for p in PROBLEM_CHILDREN)
    ],
)
def test_geojson_cover(filepath: Path) -> None:
    _test_geojson_cover(str(filepath))
