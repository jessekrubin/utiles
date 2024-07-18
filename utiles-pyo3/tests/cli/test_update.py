"""Utiles rust cli tests"""

import sqlite3
from pathlib import Path
from shutil import copyfile

from utiles.dev.testing import query_metadata_rows
from utiles.dev.testing import run_cli as _run_cli


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def _all_filepaths(dirpath: Path) -> list[str]:
    return [str(f) for f in dirpath.rglob("*") if f.is_file()]


def test_update_metadata(tmp_path: Path, test_data_root: Path) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    # copy the mbt db file to the tmp_path

    dbpath = tmp_path / "test.mbtiles"
    copyfile(_mbtiles_filepath, dbpath)
    metadata_keys_to_delete = ["format", "minzoom", "maxzoom", "tilesize", "name"]

    with sqlite3.connect(dbpath) as conn:
        cursor = conn.cursor()
        for key in metadata_keys_to_delete:
            cursor.execute(f"DELETE FROM metadata WHERE name = '{key}';")
        conn.commit()

    # get the metadata from the new mbt
    metadata_proc = _run_cli(["metadata", str(dbpath)])
    metadata = metadata_proc.parse_json
    for key in metadata_keys_to_delete:
        assert key not in metadata

    # copy file to `D:\\utiles\\test.mbtiles`
    copyfile(dbpath, "D:\\utiles\\test.mbtiles")

    # run update on the new mbt
    update_result = _run_cli(["update", str(dbpath), "--debug"])
    update_result.echo()

    assert update_result.returncode == 0

    metadata_proc_updated = _run_cli(["metadata", "--obj", str(dbpath)])
    metadata_updated = metadata_proc_updated.parse_json
    print(metadata_updated)
    assert metadata_updated["format"] == "png"
    assert metadata_updated["minzoom"] == 0
    assert metadata_updated["maxzoom"] == 4
    # assert metadata_updated["tilesize"] == 256
    # assert metadata_updated["name"] == "osm-standard.z0z4"
