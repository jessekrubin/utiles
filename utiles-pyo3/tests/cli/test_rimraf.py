"""Utiles rust cli rimraf tests"""

from __future__ import annotations

from pathlib import Path

from utiles.dev.testing import run_cli as _run_cli


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def _all_filepaths(dirpath: Path) -> list[str]:
    return [str(f) for f in dirpath.rglob("*") if f.is_file()]


class TestRimraf:
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

        # nuke the dir with rimrafer
        result = _run_cli(["rimraf", str(out_path)])

        assert result.returncode == 0

        assert not out_path.exists()
