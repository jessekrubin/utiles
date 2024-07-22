"""Utiles rust cli tests"""

import json
from pathlib import Path

from pytest import fixture

from utiles.dev.testing import run_cli as _run_cli


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def test_touch(tmp_path: Path, test_data_root: Path) -> None:
    # make a new file
    new_mbtiles = tmp_path / "new.mbtiles"
    result = _run_cli(["touch", str(new_mbtiles)])
    assert result.returncode == 0
    assert new_mbtiles.exists()
    assert new_mbtiles.is_file()
    assert new_mbtiles.suffix == ".mbtiles"

    result = _run_cli(["info", str(new_mbtiles)])
    assert result.returncode == 0
    result.print()
    parsed_data = json.loads(result.stdout)
    assert parsed_data["ntiles"] == 0
    expected_info_json = {
        "filesize": 20480,
        "mbtype": "flat",
        "ntiles": 0,
        "nzooms": 0,
        "page_count": 5,
        "page_size": 4096,
        "freelist_count": 0,
        "minzoom": None,
        "maxzoom": None,
        "zooms": [],
    }
    assert parsed_data == expected_info_json


def test_touch_db_type(tmp_path: Path, db_type: str) -> None:
    # make a new file
    new_mbtiles = tmp_path / "new.mbtiles"
    result = _run_cli(["touch", str(new_mbtiles), "--db-type", db_type])
    assert result.returncode == 0
    assert new_mbtiles.exists()
    assert new_mbtiles.is_file()
    assert new_mbtiles.suffix == ".mbtiles"

    result = _run_cli(["info", str(new_mbtiles)])
    assert result.returncode == 0
    result.print()
    parsed_data = json.loads(result.stdout)
    assert parsed_data["ntiles"] == 0
    assert parsed_data["mbtype"] == db_type


def test_touch_page_size_512(tmp_path: Path) -> None:
    # make a new file
    new_mbtiles = tmp_path / "new.mbtiles"
    result = _run_cli(["touch", str(new_mbtiles), "--page-size", "512"])
    assert result.returncode == 0
    assert new_mbtiles.exists()
    assert new_mbtiles.is_file()
    assert new_mbtiles.suffix == ".mbtiles"

    result = _run_cli(["info", "--debug", str(new_mbtiles)])
    result.print()
    assert result.returncode == 0
    parsed_data = json.loads(result.stdout)
    assert parsed_data["ntiles"] == 0
    expected_info_json = {
        "filesize": 3072,
        "mbtype": "flat",
        "ntiles": 0,
        "nzooms": 0,
        "page_count": 6,
        "page_size": 512,
        "freelist_count": 0,
        "minzoom": None,
        "maxzoom": None,
        "zooms": [],
    }
    assert parsed_data == expected_info_json


def test_touch_page_size_invalid(tmp_path: Path) -> None:
    # make a new file
    new_mbtiles = tmp_path / "new.mbtiles"
    result = _run_cli(["touch", str(new_mbtiles), "--page-size", "123"])
    assert result.returncode == 1


def test_mbtiles_info(test_data_root: Path) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    result = _run_cli(["info", str(_mbtiles_filepath)])
    assert result.returncode == 0
    parsed_data = json.loads(result.stdout)
    assert parsed_data["ntiles"] == 341

    expected_info_json = {
        "filesize": 1572864,
        "mbtype": "flat",
        "ntiles": 341,
        "nzooms": 5,
        "page_count": 384,
        "page_size": 4096,
        "freelist_count": 0,
        "minzoom": 0,
        "maxzoom": 4,
        "zooms": [
            {
                "zoom": 0,
                "ntiles": 1,
                "xmin": 0,
                "xmax": 0,
                "ymin": 0,
                "ymax": 0,
                "nbytes": 6915,
            },
            {
                "zoom": 1,
                "ntiles": 4,
                "xmin": 0,
                "xmax": 1,
                "ymin": 0,
                "ymax": 1,
                "nbytes": 24787,
            },
            {
                "zoom": 2,
                "ntiles": 16,
                "xmin": 0,
                "xmax": 3,
                "ymin": 0,
                "ymax": 3,
                "nbytes": 66425,
            },
            {
                "zoom": 3,
                "ntiles": 64,
                "xmin": 0,
                "xmax": 7,
                "ymin": 0,
                "ymax": 7,
                "nbytes": 211704,
            },
            {
                "zoom": 4,
                "ntiles": 256,
                "xmin": 0,
                "xmax": 15,
                "ymin": 0,
                "ymax": 15,
                "nbytes": 1106134,
            },
        ],
    }
    parsed_data["zooms"] = [
        {k: v for k, v in e.items() if k != "nbytes_avg"} for e in parsed_data["zooms"]
    ]
    print(parsed_data)
    assert parsed_data == expected_info_json
