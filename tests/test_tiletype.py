from pathlib import Path
from typing import Union

import pytest

import utiles

Extensions = Union[str, bool]

PWD = Path(__file__).parent

def _repo_root() -> Path:
    _root = PWD
    for _i in  range(5):
        _root = _root.parent
        if (_root / ".github").is_dir():
            return _root
    raise RuntimeError("Could not find repo root")

REPO_ROOT = _repo_root()

# go up and find dir with sub dir ".github"


def tiletype(buffer: bytes) -> Extensions:
    if (
        buffer[0] == 0x89
        and buffer[1] == 0x50
        and buffer[2] == 0x4E
        and buffer[3] == 0x47
        and buffer[4] == 0x0D
        and buffer[5] == 0x0A
        and buffer[6] == 0x1A
        and buffer[7] == 0x0A
    ):
        return "png"
    elif (
        buffer[0] == 0xFF
        and buffer[1] == 0xD8
        and buffer[-2] == 0xFF
        and buffer[-1] == 0xD9
    ):
        return "jpg"
    elif (
        buffer[0] == 0x47
        and buffer[1] == 0x49
        and buffer[2] == 0x46
        and buffer[3] == 0x38
        and (buffer[4] == 0x39 or buffer[4] == 0x37)
        and buffer[5] == 0x61
    ):
        return "gif"
    elif (
        buffer[0] == 0x52
        and buffer[1] == 0x49
        and buffer[2] == 0x46
        and buffer[3] == 0x46
        and buffer[8] == 0x57
        and buffer[9] == 0x45
        and buffer[10] == 0x42
        and buffer[11] == 0x50
    ):
        return "webp"
    # deflate: recklessly assumes contents are PBF.
    elif buffer[0] == 0x78 and buffer[1] == 0x9C:
        return "pbf"
    # gzip: recklessly assumes contents are PBF.
    elif buffer[0] == 0x1F and buffer[1] == 0x8B:
        return "pbfgz"
    # if buffer starts with '{' or '[' assume JSON
    elif buffer[0] == 0x7B or buffer[0] == 0x5B:
        return "json"
    return False


TEST_TILES = (REPO_ROOT / "test-data" / "tile-types").glob("**/*")

TEST_TILES_BYTES = [(str(f.name), f.read_bytes()) for f in TEST_TILES]

TEST_TILE_NAME2TYPE = {
    "0.gif": "gif",
    "0.jpeg": "jpg",
    "0.png": "png",
    "0.vector.pbf": "pbf",
    "0.vector.pbfz": "pbfgz",
    "0.webp": "webp",
    "gif-990x1050.gif": "gif",
    "jpg-640x400.jpg": "jpg",
    "png-640x400.png": "png",
    "tux.webp": "webp",
    "tux_alpha.webp": "webp",
    "unknown.txt": False,
    "webp-550x368.webp": "webp",
    "tile-arr.json": "json",
    "tile-obj.json": "json",
}

def test_found_test_files() -> None:
    assert len(TEST_TILES_BYTES) == len(TEST_TILE_NAME2TYPE)


@pytest.mark.parametrize(
    "tile",
    TEST_TILES_BYTES,
)
def test_tiletype(
    tile: tuple[str, bytes],
) -> None:
    filename, buffer = tile
    expected = TEST_TILE_NAME2TYPE[filename]
    ttype = tiletype(buffer)
    if filename == "unknown.txt":
        assert ttype is False
    else:
        assert ttype == expected


@pytest.mark.parametrize(
    "tile",
    TEST_TILES_BYTES,
)
def test_tiletype_rs(
    tile: tuple[str, bytes],
) -> None:
    filename, buffer = tile
    expected = TEST_TILE_NAME2TYPE[filename]
    ttype = utiles.tiletype_str(buffer)
    if filename == "unknown.txt":
        assert ttype is False or ttype == "unknown"  # type: ignore
    else:
        assert ttype == expected


# @pytest.mark.parametrize(
#     "tile",
#     TEST_TILES_BYTES,
# )
# def test_benchmark_tiletype_py(
#     tile: tuple[str, bytes],
#     benchmark,
# ):
#     filename, buffer = tile
#     benchmark(tiletype, buffer)
#
#
# @pytest.mark.parametrize(
#     "tile",
#     TEST_TILES_BYTES,
# )
# def test_benchmark_tiletype_rs(
#     tile: tuple[str, bytes],
#     benchmark,
# ):
#     filename, buffer = tile
#     benchmark(utiles.tiletype_str, buffer)
