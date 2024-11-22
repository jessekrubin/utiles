from __future__ import annotations

from pathlib import Path

from utiles.dev.testing import run_cli as _run_cli


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def test_agg_hash_md5(test_data_root: Path) -> None:
    """
    Md5 should be: '3A9279283D4D6B5B12362E3A76AF7201'
    Verified by the martin people
    """
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    agg_hash_result = _run_cli(["agg-hash", str(_mbtiles_filepath)])
    assert agg_hash_result.returncode == 0
    assert agg_hash_result.parse_json["hash"] == "3A9279283D4D6B5B12362E3A76AF7201"


def test_agg_hash_md5_bbox(test_data_root: Path) -> None:
    """
    Md5 should be: 'E0DF65DB0BE3C50FB566CCD78B7E88ED'
    Verified by the martin people
    """
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    agg_hash_result = _run_cli(
        ["agg-hash", str(_mbtiles_filepath), "--bbox", "1,1,179,80"]
    )
    assert agg_hash_result.returncode == 0
    assert agg_hash_result.parse_json["hash"] == "E0DF65DB0BE3C50FB566CCD78B7E88ED"
    assert agg_hash_result.parse_json["ntiles"] == 78
