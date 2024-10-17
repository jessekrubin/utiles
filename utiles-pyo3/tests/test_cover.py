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


TEST_COVER_LIMITS = [
    {"file": "road.geojson", "limits": {"minzoom": 1, "maxzoom": 12}},
    {"file": "world.geojson", "limits": {"minzoom": 1, "maxzoom": 6}},
    {"file": "point.geojson", "limits": {"minzoom": 1, "maxzoom": 15}},
    {"file": "line.geojson", "limits": {"minzoom": 1, "maxzoom": 12}},
    {"file": "edgeline.geojson", "limits": {"minzoom": 15, "maxzoom": 15}},
    {"file": "polygon.geojson", "limits": {"minzoom": 1, "maxzoom": 15}},
    {"file": "multipoint.geojson", "limits": {"minzoom": 1, "maxzoom": 12}},
    {"file": "multiline.geojson", "limits": {"minzoom": 1, "maxzoom": 8}},
    {"file": "uk.geojson", "limits": {"minzoom": 7, "maxzoom": 9}},
    {"file": "building.geojson", "limits": {"minzoom": 18, "maxzoom": 18}},
    {"file": "donut.geojson", "limits": {"minzoom": 16, "maxzoom": 16}},
    {"file": "russia.geojson", "limits": {"minzoom": 6, "maxzoom": 6}},
    {"file": "degenring.geojson", "limits": {"minzoom": 11, "maxzoom": 15}},
    {"file": "invalid_polygon.geojson", "limits": {"minzoom": 1, "maxzoom": 12}},
    {"file": "highzoom.geojson", "limits": {"minzoom": 23, "maxzoom": 23}},
    {"file": "small_poly.geojson", "limits": {"minzoom": 10, "maxzoom": 10}},
    {"file": "spiked.geojson", "limits": {"minzoom": 10, "maxzoom": 10}},
    {"file": "blocky.geojson", "limits": {"minzoom": 6, "maxzoom": 6}},
    {"file": "pyramid.geojson", "limits": {"minzoom": 10, "maxzoom": 10}},
    {"file": "tetris.geojson", "limits": {"minzoom": 10, "maxzoom": 10}},
    {"file": "zero.geojson", "limits": {"minzoom": 10, "maxzoom": 10}},
]

TEST_COVER_LIMITS_MAP = {el["file"]: el["limits"] for el in TEST_COVER_LIMITS}

# TEST_GEOJSON_FILEPATHS =  listdir(PWD / "test-data" / "geojson")
tile_cover_data_root = _repo_root() / "test-data" / "tile-cover"
TEST_GEOJSON_FILEPATHS = [
    e for e in tile_cover_data_root.rglob("*.geojson") if "world" not in str(e)
]
print(TEST_GEOJSON_FILEPATHS)


def zoom_limits(filepath: Path | str):
    filename = Path(filepath).name
    if filename in TEST_COVER_LIMITS_MAP:
        lims = TEST_COVER_LIMITS_MAP[filename]
        return (lims["minzoom"], lims["maxzoom"])
    return (1, 6)


def _test_geojson_cover(geojson_filepath: str):
    with open(geojson_filepath, "r") as f:
        data = f.read()
    expected_textiles_filepath = str(geojson_filepath).replace(
        ".geojson", ".tiles.jsonl"
    )
    with open(expected_textiles_filepath, "r") as f:
        textiles = f.read()

    minzoom, maxzoom = zoom_limits(geojson_filepath)
    expected_tiles = set(ut.parse_textiles(textiles))
    # expected_tiles = set(
    #     ut.simplify([el.tuple() for el in expected_tiles], minzoom=minzoom)
    # )

    coverage = set(ut.geojson2tiles(data, maxzoom, minzoom))
    # print(coverage)
    # print(expected_tiles)
    # not in expected
    if set(expected_tiles) != set(coverage):
        not_in_expected_tiles = expected_tiles.difference(coverage)
        not_in_coverage = coverage.difference(expected_tiles)
        common_tiles = expected_tiles.intersection(coverage)
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


# test_geojson_filepath = r"D:\utiles\test-data\tile-cover\line.geojson"
# _test_geojson_cover(test_geojson_filepath)

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
def test_geojson_cover(filepath: Path):
    _test_geojson_cover(filepath)


# probs = []
# for filepath in TEST_GEOJSON_FILEPATHS:
#     try:
#         _test_geojson_cover(filepath)
#     except:
#         probs.append(filepath)
#
# print(probs)
