"""Utiles rust cli tests"""

import json
from pathlib import Path

from utiles.dev.testing import run_cli as _run_cli


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def _all_filepaths(dirpath: Path) -> list[str]:
    return [str(f) for f in dirpath.rglob("*") if f.is_file()]


class TestCopyPyramid:
    def test_mbtiles2pyramid(self, tmp_path: Path, test_data_root: Path) -> None:
        _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
        out_path = tmp_path / "osm-pyramid"
        # copy mbtiles to tile pyramid
        result = _run_cli(["cp", str(_mbtiles_filepath), str(out_path)])
        assert result.returncode == 0
        assert out_path.exists()
        assert out_path.is_dir()
        assert len(_all_filepaths(out_path)) > 0
        all_paths = _all_filepaths(out_path)
        png_paths = [p for p in all_paths if p.endswith(".png")]
        assert len(png_paths) > 0
        assert len(png_paths) == 341

        # copy pyramid to new mbtiles
        out_mbtiles = tmp_path / "osm-pyramid.mbtiles"
        result = _run_cli(["cp", str(out_path), str(out_mbtiles)])
        assert result.returncode == 0
        assert out_mbtiles.exists()
        assert out_mbtiles.is_file()
        assert out_mbtiles.suffix == ".mbtiles"
        assert out_mbtiles.stat().st_size > 0

        db_info_completed_proc = _run_cli(["info", str(out_mbtiles)])
        assert db_info_completed_proc.returncode == 0
        parsed_data = json.loads(db_info_completed_proc.stdout)
        assert parsed_data["ntiles"] == 341

    def test_mbtiles2pyramid_bbox(self, tmp_path: Path, test_data_root: Path) -> None:
        _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
        out_path = tmp_path / "osm-pyramid"
        # copy mbtiles to tile pyramid with bbox of -180,0,0,90 (top left quadrant)
        result = _run_cli(
            ["cp", str(_mbtiles_filepath), str(out_path), "--bbox", "-180,0,0,90"]
        )
        assert result.returncode == 0
        assert out_path.exists()
        assert out_path.is_dir()
        assert len(_all_filepaths(out_path)) > 0
        all_paths = _all_filepaths(out_path)
        png_paths = [p for p in all_paths if p.endswith(".png")]
        assert len(png_paths) > 0
        assert len(png_paths) == 86

        # copy pyramid to new mbtiles
        out_mbtiles = tmp_path / "osm-pyramid.mbtiles"
        result = _run_cli(["cp", str(out_path), str(out_mbtiles)])
        assert result.returncode == 0
        assert out_mbtiles.exists()
        assert out_mbtiles.is_file()
        assert out_mbtiles.suffix == ".mbtiles"
        assert out_mbtiles.stat().st_size > 0

        db_info_completed_proc = _run_cli(["info", str(out_mbtiles)])
        assert db_info_completed_proc.returncode == 0
        parsed_data = json.loads(db_info_completed_proc.stdout)
        assert parsed_data["ntiles"] == 86

    def test_mbtiles2pyramid_minzoom(
        self, tmp_path: Path, test_data_root: Path
    ) -> None:
        _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
        out_path = tmp_path / "osm-pyramid"
        # copy mbtiles to tile pyramid with bbox of -180,0,0,90 (top left quadrant)
        result = _run_cli(
            ["cp", str(_mbtiles_filepath), str(out_path), "--minzoom", "3"]
        )
        assert result.returncode == 0
        assert out_path.exists()
        assert out_path.is_dir()
        assert len(_all_filepaths(out_path)) > 0
        all_paths = _all_filepaths(out_path)
        png_paths = [p for p in all_paths if p.endswith(".png")]
        assert len(png_paths) > 0
        assert len(png_paths) == 320

        # copy pyramid to new mbtiles
        out_mbtiles = tmp_path / "osm-pyramid.mbtiles"
        result = _run_cli(["cp", str(out_path), str(out_mbtiles)])
        assert result.returncode == 0
        assert out_mbtiles.exists()
        assert out_mbtiles.is_file()
        assert out_mbtiles.suffix == ".mbtiles"
        assert out_mbtiles.stat().st_size > 0

        db_info_completed_proc = _run_cli(["info", str(out_mbtiles)])
        assert db_info_completed_proc.returncode == 0
        parsed_data = json.loads(db_info_completed_proc.stdout)
        assert parsed_data["ntiles"] == 320
