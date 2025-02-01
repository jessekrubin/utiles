"""Utiles rust cli tests"""

from __future__ import annotations

import dataclasses
import typing as t
from pathlib import Path

import pytest

from utiles.dev.testing import run_cli as _run_cli


@dataclasses.dataclass
class DbCopyCfg:
    db_type: str
    stream: bool

    def argv(self) -> list[str]:
        return (
            ["--dbtype", self.db_type, "--stream"]
            if self.stream
            else ["--dbtype", self.db_type]
        )


DB_COPY_CONFIGS = [
    *(DbCopyCfg(db_type=e, stream=False) for e in ["flat", "hash", "norm"]),
    *(DbCopyCfg(db_type=e, stream=True) for e in ["flat", "hash", "norm"]),
]


@pytest.fixture(params=DB_COPY_CONFIGS)
def db_copy_cfg(request: pytest.FixtureRequest) -> DbCopyCfg:
    """Fixture for testing different db/schema types"""
    if isinstance(request.param, DbCopyCfg):
        return request.param
    else:
        raise ValueError("Should not happen - request.param is not a DbCopyCfg")


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def _all_filepaths(dirpath: Path) -> list[str]:
    return [str(f) for f in dirpath.rglob("*") if f.is_file()]


# ============================================================================
def test_copy_mbtiles(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    copy_result = _run_cli(
        ["cp", str(_mbtiles_filepath), str(out_path), *db_copy_cfg.argv()]
    )

    assert copy_result.returncode == 0
    info_result = _run_cli(["info", str(out_path)])
    info_dict = info_result.parse_json
    expected_key_values = {"ntiles": 341, "nzooms": 5, "minzoom": 0, "maxzoom": 4}
    for k, v in expected_key_values.items():
        assert info_dict[k] == v

    assert db_copy_cfg.db_type == info_dict["mbtype"]


def test_copy_mbtiles_zooms(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    copy_result = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--minzoom",
            "3",
        ]
    )

    assert copy_result.returncode == 0
    info_result = _run_cli(["info", str(out_path)])
    info_dict = info_result.parse_json
    expected_key_values = {
        "ntiles": (((2 << (3 - 1)) ** 2) + ((2 << (4 - 1)) ** 2)),
        "nzooms": 2,
        "minzoom": 3,
        "maxzoom": 4,
    }
    for k, v in expected_key_values.items():
        assert info_dict[k] == v

    assert db_copy_cfg.db_type == info_dict["mbtype"]


def test_copy_mbtiles_conflict(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    copy_result_a = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--minzoom",
            "3",
        ]
    )
    assert copy_result_a.returncode == 0

    # no specifying of the zooms...
    copy_result_b = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
        ]
    )

    assert copy_result_b.returncode != 0


def test_copy_mbtiles_conflict_with_strategy(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    copy_result_a = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--minzoom",
            "3",
        ]
    )

    assert copy_result_a.returncode == 0

    # no specifying of the zooms...
    copy_result_b = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--debug",
            "--conflict",
            "ignore",
        ]
    )
    assert copy_result_b.returncode == 0


def test_copy_mbtiles_conflict_with_strategy_not_overlapping(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    copy_result_a = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--minzoom",
            "3",
        ]
    )

    assert copy_result_a.returncode == 0

    # no specifying of the zooms...
    copy_result_b = _run_cli(
        [
            "cp",
            str(_mbtiles_filepath),
            str(out_path),
            *db_copy_cfg.argv(),
            "--debug",
            "--maxzoom",
            "2",
        ]
    )
    assert copy_result_b.returncode == 0


def test_copy_mbtiles_bbox(
    tmp_path: Path, test_data_root: Path, db_copy_cfg: DbCopyCfg
) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    out_path = tmp_path / "copied.mbtiles"
    west_half_o_world_args = [
        "cp",
        str(_mbtiles_filepath),
        str(out_path),
        *db_copy_cfg.argv(),
        "--minzoom",
        "3",
        "--maxzoom",
        "4",
        "--bbox",
        "-180,-90,0,90",
    ]
    # print(" ".join(west_half_o_world_args))
    copy_result_a = _run_cli(west_half_o_world_args)

    assert copy_result_a.returncode == 0

    info_result = _run_cli(["info", str(out_path)])
    info_dict = info_result.parse_json
    expected_ntiles_total = ((2 << (3 - 1)) ** 2) + ((2 << (4 - 1)) ** 2)
    assert info_dict["ntiles"] == expected_ntiles_total // 2

    # no specifying of the zooms...
    east_half_o_world_args = [
        "cp",
        str(_mbtiles_filepath),
        str(out_path),
        *db_copy_cfg.argv(),
        "--minzoom",
        "3",
        "--maxzoom",
        "4",
        "--bbox",
        "0,-90,180,90",
    ]
    copy_result_b = _run_cli(east_half_o_world_args)
    assert copy_result_b.returncode == 0

    info_result_final = _run_cli(["info", str(out_path)])
    info_dict_final = info_result_final.parse_json
    # print(info_dict_final)
    assert info_dict_final["ntiles"] == expected_ntiles_total
