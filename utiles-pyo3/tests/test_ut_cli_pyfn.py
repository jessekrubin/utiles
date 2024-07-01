from pathlib import Path

import utiles as ut


def _osm_standard_z0z4_mbtiles(test_data: Path) -> Path:
    return test_data / "mbtiles" / "osm-standard.z0z4.mbtiles"


def test_mbtiles_info_double_call(test_data_root: Path) -> None:
    _mbtiles_filepath = _osm_standard_z0z4_mbtiles(test_data_root)
    args = ["info", str(_mbtiles_filepath)]
    r1 = ut.ut_cli(args)
    assert r1 == 0
    r2 = ut.ut_cli(args)
    assert r2 == 0


if __name__ == "__main__":
    test_mbtiles_info_double_call(Path(__file__).parent.parent.parent / "test-data")
