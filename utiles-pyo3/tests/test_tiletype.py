from __future__ import annotations

from pathlib import Path

import pytest
from pytest_benchmark.fixture import BenchmarkFixture

import utiles

PWD = Path(__file__).parent


def _repo_root() -> Path:
    _root = PWD
    for _i in range(5):
        _root = _root.parent
        if (_root / ".github").is_dir():
            return _root
    msg = "Could not find repo root"
    raise RuntimeError(msg)


REPO_ROOT = _repo_root()


# go up and find dir with sub dir ".github"


def is_mvt_like(buffer: bytes) -> bool:
    if len(buffer) < 2:
        return False

    i = 0

    while i < len(buffer):
        key, wire_type = buffer[i] >> 3, buffer[i] & 0x07
        i += 1

        if key == 0 or key > 15:
            return False

        if wire_type == 0:
            while i < len(buffer) and buffer[i] & 0x80 != 0:
                i += 1
            i += 1
        elif wire_type == 1:
            i += 8  # 64-bit
        elif wire_type == 2:
            length = 0
            shift = 0
            while i < len(buffer) and buffer[i] & 0x80 != 0:
                length |= (buffer[i] & 0x7F) << shift
                shift += 7
                i += 1
            if i < len(buffer):
                length |= buffer[i] << shift
            i += 1
            i += length
        elif wire_type == 5:
            i += 4  # 32-bit
        else:
            return False

        if i > len(buffer):
            return False

    return True


def tiletype(buffer: bytes) -> str | bool:
    if buffer.startswith(b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a"):
        return "png"
    # if (
    #     buffer[0] == 0x89
    #     and buffer[1] == 0x50
    #     and buffer[2] == 0x4E
    #     and buffer[3] == 0x47
    #     and buffer[4] == 0x0D
    #     and buffer[5] == 0x0A
    #     and buffer[6] == 0x1A
    #     and buffer[7] == 0x0A
    # ):
    #     return "png"
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
        return "pbf.zlib"
    # gzip: recklessly assumes contents are PBF.
    elif buffer[0] == 0x1F and buffer[1] == 0x8B:
        return "pbf.gz"
    # zstd: recklessly assumes contents are PBF.
    elif buffer[0] == 0x28 and buffer[1] == 0xB5:
        return "pbf.zst"
    # if buffer starts with '{' or '[' assume JSON
    elif buffer[0] == 0x7B or buffer[0] == 0x5B:
        return "json"
    if is_mvt_like(buffer):
        return "pbf"
    return False


TEST_TILES = (REPO_ROOT / "test-data" / "tile-types").glob("**/*")

TEST_TILES_BYTES = [
    (str(f.name), f.read_bytes())
    for f in TEST_TILES
    # if f.name != "0.vector.pbf"
]

TEST_TILE_NAME2TYPE = {
    "0.gif": "gif",
    "0.jpeg": "jpg",
    "0.png": "png",
    # TODO figure out how to handle uncompressed PBF
    "0.vector.pbf": "pbf",
    "0.vector.pbf.zst": "pbf.zst",
    "0.vector.pbf.zlib": "pbf.zlib",
    "0.vector.pbf.gz": "pbf.gz",
    "0.webp": "webp",
    "gif-990x1050.gif": "gif",
    "jpg-640x400.jpg": "jpg",
    "png-640x400.png": "png",
    "tux.webp": "webp",
    "tux_alpha.webp": "webp",
    "unknown.txt": "unknown",
    "webp-550x368.webp": "webp",
    "tile-arr.json": "json",
    "tile-obj.json": "json",
}

TEST_TILE_NAME2ENCODING = {
    "0.gif": "internal",
    "0.jpeg": "internal",
    "0.png": "internal",
    "0.vector.pbf": "uncompressed",
    "0.vector.pbf.zst": "zstd",
    "0.vector.pbf.zlib": "zlib",
    "0.vector.pbf.gz": "gzip",
    "0.webp": "internal",
    "gif-990x1050.gif": "internal",
    "jpg-640x400.jpg": "internal",
    "png-640x400.png": "internal",
    "tux.webp": "internal",
    "tux_alpha.webp": "internal",
    "unknown.txt": "uncompressed",
    "webp-550x368.webp": "internal",
    "tile-arr.json": "uncompressed",
    "tile-obj.json": "uncompressed",
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

    ttype = utiles.TileType.from_bytes(buffer)
    expected_format = expected.split(".")[0]
    assert ttype.format == expected_format
    ttype_str = utiles.tiletype_str(buffer)
    if filename == "unknown.txt":
        assert ttype_str is False or ttype_str == "unknown"  # type: ignore
    else:
        assert ttype_str == expected


@pytest.mark.parametrize(
    "tile",
    TEST_TILES_BYTES,
)
def test_tiletype_obj(
    tile: tuple[str, bytes],
) -> None:
    filename, buffer = tile
    ttype = utiles.TileType.from_bytes(buffer)
    expected = TEST_TILE_NAME2TYPE[filename]
    assert ttype.format == expected.split(".")[0]
    assert ttype.encoding == TEST_TILE_NAME2ENCODING[filename]


@pytest.mark.parametrize(
    "tile",
    TEST_TILES_BYTES,
)
def test_benchmark_tiletype_py(
    tile: tuple[str, bytes],
    benchmark: BenchmarkFixture,
) -> None:
    _filename, buffer = tile
    benchmark(tiletype, buffer)


@pytest.mark.parametrize(
    "tile",
    TEST_TILES_BYTES,
)
def test_benchmark_tiletype_rs(
    tile: tuple[str, bytes],
    benchmark: BenchmarkFixture,
) -> None:
    _filename, buffer = tile
    benchmark(utiles.tiletype_str, buffer)
