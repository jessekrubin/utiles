"""Utiles rust cli tests"""
import json
import sys
from json import dumps as stringify
from subprocess import CompletedProcess, run

import pytest

from utiles.dev.testing import run_cli as _run_cli


def _run_cli_old(
    args: list[str] | None,
    input: str | None = None,
) -> CompletedProcess[str]:
    _python = sys.executable
    _args = args or []
    res = run(
        [_python, "-m", "utiles.cli", *_args],
        input=input,
        capture_output=True,
        text=True,
        shell=False,  # noqa: S603
    )
    return res


def test_rust_cli_help() -> None:
    res = _run_cli(["--help"])
    assert "(rust)" in res.stdout


class TestTiles:
    def test_cli_tiles_bad_bounds(self) -> None:
        """Bounds of len 3 are bad."""
        result = _run_cli(["tiles", "14"], "[-105, 39.99, -104.99]")
        assert result.returncode != 0

    def test_cli_tiles_no_bounds(self) -> None:
        result = _run_cli(["tiles", "14"], "[-105, 39.99, -104.99, 40]")
        assert result.returncode == 0
        assert result.stdout == "[3413, 6202, 14]\n[3413, 6203, 14]\n"

    #
    def test_cli_tiles_multi_bounds(self) -> None:
        """A LF-delimited sequence can be used as input."""
        result = _run_cli(
            ["tiles", "14"], "[-105, 39.99, -104.99, 40]\n[-105, 39.99, -104.99, 40]"
        )
        assert result.returncode == 0
        assert len(result.stdout.strip().split("\n")) == 4

    def test_cli_tiles_multi_bounds_seq(self) -> None:
        """A JSON text sequence can be used as input."""
        result = _run_cli(
            ["--debug", "tiles", "14"],
            "\x1e\n[-105, 39.99, -104.99, 40]\n\x1e\n[-105, 39.99, -104.99, 40]",
        )
        assert result.returncode == 0
        assert len(result.stdout.strip().split("\n")) == 4

    def test_rust_cli_tiles_seq(self) -> None:
        result = _run_cli(["tiles", "14", "--seq", "[14.0859, 5.798]"])
        # runner = CliRunner()
        # result = runner.invoke(cli, ["tiles", "14", "--seq",
        #                              "[14.0859, 5.798]"
        #                              ],)
        # print(result)
        assert result.returncode == 0
        assert result.stdout == "\x1e\n[8833, 7927, 14]\n"

    def test_cli_tiles_points(self) -> None:
        result = _run_cli(["tiles", "14"], "[14.0859, 5.798]")
        # j
        # runner = CliRunner()
        # result = runner.invoke(cli, ["tiles", "14"], "[14.0859, 5.798]")
        assert result.returncode == 0
        assert result.stdout == "[8833, 7927, 14]\n"

    def test_cli_tiles_point_geojson(self) -> None:
        result = _run_cli(
            ["tiles", "14"], '{"type":"Point","coordinates":[14.0859, 5.798]}'
        )
        assert result.returncode == 0
        assert result.stdout == "[8833, 7927, 14]\n"

    def test_cli_tiles_implicit_stdin(self) -> None:
        # result = _run_cli(["tiles", "14"], "[14.0859, 5.798]")
        # assert result.returncode == 0
        # assert result.stdout == "[8833, 7927, 14]\n"
        # runner = CliRunner()
        result = _run_cli(["tiles", "14"], "[-105, 39.99, -104.99, 40]")
        assert result.returncode == 0
        assert result.stdout == "[3413, 6202, 14]\n[3413, 6203, 14]\n"

    def test_cli_tiles_arg(self) -> None:
        result = _run_cli(["tiles", "14", "[-105, 39.99, -104.99, 40]"])
        assert result.returncode == 0
        assert result.stdout == "[3413, 6202, 14]\n[3413, 6203, 14]\n"

    def test_cli_tiles_geojson(self) -> None:
        collection = stringify(
            {
                "features": [
                    {
                        "geometry": {
                            "coordinates": [
                                [
                                    [-105.46875, 39.909736],
                                    [-105.46875, 40.446947],
                                    [-104.765625, 40.446947],
                                    [-104.765625, 39.909736],
                                    [-105.46875, 39.909736],
                                ]
                            ],
                            "type": "Polygon",
                        },
                        "id": "(106, 193, 9)",
                        "properties": {"title": "XYZ tile (106, 193, 9)"},
                        "type": "Feature",
                    }
                ],
                "type": "FeatureCollection",
            }
        )
        result = _run_cli(["tiles", "9"], collection)
        assert result.returncode == 0
        assert result.stdout == "[106, 193, 9]\n[106, 194, 9]\n"


class TestQuadkey:
    def test_cli_quadkey_from_tiles(self) -> None:
        result = _run_cli(["quadkey"], "[486, 332, 10]\n[6826, 12415, 15]")
        assert result.returncode == 0
        assert result.stdout == "0313102310\n023101012323232\n"

    def test_cli_quadkey_from_quadkeys(self) -> None:
        result = _run_cli(["quadkey"], "0313102310\n023101012323232")
        assert result.returncode == 0
        assert result.stdout == "[486, 332, 10]\n[6826, 12415, 15]\n"

    def test_cli_quadkey_from_mixed(self) -> None:
        result = _run_cli(["quadkey"], "0313102310\n[6826, 12415, 15]")
        assert result.returncode == 0
        assert result.stdout == "[486, 332, 10]\n023101012323232\n"

    @pytest.mark.skip(reason="not implemented")
    def test_cli_quadkey_failure(self) -> None:
        """Abort when an invalid quadkey is passed"""
        result = _run_cli(["quadkey"], "lolwut")
        assert result.returncode != 0
        assert "lolwut" in result.stdout


class TestBoundingTile:
    def test_cli_bounding_tile_bad_bounds(self) -> None:
        """Bounds of len 3 are bad."""
        result = _run_cli(["bounding-tile"], "[-105, 39.99, -104.99]")
        assert result.returncode != 0

    def test_cli_bounding_tile(self) -> None:
        result = _run_cli(["bounding-tile"], "[-105, 39.99, -104.99, 40]")
        assert result.returncode == 0
        assert result.stdout == "[1706, 3101, 13]\n"

    def test_cli_bounding_tile_bbox(self) -> None:
        result = _run_cli(["bounding-tile"], '{"bbox": [-105, 39.99, -104.99, 40]}')
        assert result.returncode == 0
        assert result.stdout == "[1706, 3101, 13]\n"

    def test_cli_bounding_tile2(self) -> None:
        result = _run_cli(["bounding-tile"], "[-105, 39.99]")
        assert result.returncode == 0

    def test_cli_multi_bounding_tile(self) -> None:
        """A JSON text sequence can be used as input."""
        result = _run_cli(
            ["bounding-tile"], "[-105, 39.99, -104.99, 40]\n[-105, 39.99, -104.99, 40]"
        )
        assert result.returncode == 0
        assert len(result.stdout.strip().split("\n")) == 2

    def test_cli_multi_bounding_tile_seq(self) -> None:
        """A JSON text sequence can be used as input."""
        result = _run_cli(
            ["bounding-tile"],
            "\x1e\n[-105, 39.99, -104.99, 40]\n\x1e\n[-105, 39.99, -104.99, 40]",
        )
        assert result.returncode == 0
        assert len(result.stdout.strip().split("\n")) == 2

    @pytest.mark.skip(reason="I dont think this is correct")
    def test_cli_tiles_bounding_tiles_z0(self) -> None:
        result = _run_cli(["bounding-tile"], "[-1, -1, 1, 1]")
        assert result.returncode == 0
        assert result.stdout == "[0, 0, 0]\n"

    @pytest.mark.skip(reason="I dont think this is correct either")
    def test_cli_tiles_bounding_tiles_seq(self) -> None:
        result = _run_cli(["bounding-tile", "--seq"], "[-1, -1, 1, 1]")
        assert result.returncode == 0
        assert result.stdout == "\x1e\n[0, 0, 0]\n"

    def test_cli_bounding_tile_geojson(self) -> None:
        collection_dict = {
            "features": [
                {
                    "geometry": {
                        "coordinates": [
                            [
                                [-105.46875, 39.909736],
                                [-105.46875, 40.446947],
                                [-104.765625, 40.446947],
                                [-104.765625, 39.909736],
                                [-105.46875, 39.909736],
                            ]
                        ],
                        "type": "Polygon",
                    },
                    "id": "(106, 193, 9)",
                    "properties": {"title": "XYZ tile (106, 193, 9)"},
                    "type": "Feature",
                }
            ],
            "type": "FeatureCollection",
        }
        collection = json.dumps(collection_dict)
        result = _run_cli(["bounding-tile"], collection)
        assert result.returncode == 0
        assert result.stdout == "[26, 48, 7]\n"


class TestNeighbors:
    def test_cli_neighbors(self) -> None:
        result = _run_cli(["neighbors"], "[243, 166, 9]")
        assert result.returncode == 0

        tiles_lines = result.stdout.strip().split("\n")
        tiles = [tuple(json.loads(t)) for t in tiles_lines]
        assert len(tiles) == 8

        # We do not provide ordering guarantees
        # tiles = set([tuple(t) for t in tiles])
        tiles_set = set(tiles)
        assert (243, 166, 9) not in tiles_set, "input not in neighbors"

        assert (243 - 1, 166 - 1, 9) in tiles_set
        assert (243 - 1, 166 + 0, 9) in tiles_set
        assert (243 - 1, 166 + 1, 9) in tiles_set
        assert (243 + 0, 166 - 1, 9) in tiles_set
        assert (243 + 0, 166 + 1, 9) in tiles_set
        assert (243 + 1, 166 - 1, 9) in tiles_set
        assert (243 + 1, 166 + 0, 9) in tiles_set
        assert (243 + 1, 166 + 1, 9) in tiles_set


class TestParent:
    def test_cli_parent_failure(self) -> None:
        """[0, 0, 0] has no parent"""
        result = _run_cli(["parent"], "[0, 0, 0]")
        assert result.returncode != 0

    def test_cli_parent(self) -> None:
        result = _run_cli(["parent"], "[486, 332, 10]\n[486, 332, 10]")
        assert result.returncode == 0
        assert result.stdout == "[243, 166, 9]\n[243, 166, 9]\n"

    def test_cli_parent_depth(self) -> None:
        result = _run_cli(["parent", "--depth", "2"], "[486, 332, 10]")
        assert result.returncode == 0
        assert result.stdout == "[121, 83, 8]\n"

    def test_cli_parent_multidepth(self) -> None:
        result = _run_cli(["parent", "--depth", "2"], "[486, 332, 10]\n[121, 83, 8]")
        assert result.returncode == 0
        assert result.stdout == "[121, 83, 8]\n[30, 20, 6]\n"


class TestChildren:
    def test_cli_children(self) -> None:
        result = _run_cli(["children"], "[243, 166, 9]")
        assert result.returncode == 0
        assert (
            result.stdout
            == "[486, 332, 10]\n[487, 332, 10]\n[487, 333, 10]\n[486, 333, 10]\n"
        )


# ===================
# SHAPES TESTS (TODO)
# ===================


class TestShapes:
    def test_cli_shapes_failure(self) -> None:
        result = _run_cli(["shapes"], "0")
        assert result.returncode != 0

    def test_cli_shapes(self) -> None:
        result = _run_cli(["shapes", "--precision", "6"], "[106, 193, 9]")
        assert result.returncode == 0
        expected = {
            "bbox": [-105.46875, 39.909736, -104.765625, 40.446947],
            "geometry": {
                "coordinates": [
                    [
                        [-105.46875, 39.909736],
                        [-105.46875, 40.446947],
                        [-104.765625, 40.446947],
                        [-104.765625, 39.909736],
                        [-105.46875, 39.909736],
                    ]
                ],
                "type": "Polygon",
            },
            "id": "(106, 193, 9)",
            "properties": {"title": "XYZ tile (106, 193, 9)"},
            "type": "Feature",
        }
        assert json.loads(result.stdout) == expected

    def test_cli_shapes_arg(self) -> None:
        # runner = CliRunner()
        # result = runner.invoke(cli, ["shapes", "[106, 193, 9]", "--precision", "6"])
        result = _run_cli(["shapes", "[106, 193, 9]", "--precision", "6"])
        assert result.returncode == 0
        result_output_json = json.loads(result.stdout)

        expected_dict = {
            "bbox": [-105.46875, 39.909736, -104.765625, 40.446947],
            "geometry": {
                "coordinates": [
                    [
                        [-105.46875, 39.909736],
                        [-105.46875, 40.446947],
                        [-104.765625, 40.446947],
                        [-104.765625, 39.909736],
                        [-105.46875, 39.909736],
                    ]
                ],
                "type": "Polygon",
            },
            "id": "(106, 193, 9)",
            "properties": {"title": "XYZ tile (106, 193, 9)"},
            "type": "Feature",
        }

        assert result_output_json == expected_dict

    def test_cli_shapes_buffer(self) -> None:
        result = _run_cli(
            ["shapes", "[106, 193, 9]", "--buffer", "1.0", "--precision", "6"]
        )
        assert result.returncode == 0
        expected = {
            "bbox": [-106.46875, 38.909736, -103.765625, 41.446947],
            "geometry": {
                "coordinates": [
                    [
                        [-106.46875, 38.909736],
                        [-106.46875, 41.446947],
                        [-103.765625, 41.446947],
                        [-103.765625, 38.909736],
                        [-106.46875, 38.909736],
                    ]
                ],
                "type": "Polygon",
            },
            "id": "(106, 193, 9)",
            "properties": {"title": "XYZ tile (106, 193, 9)"},
            "type": "Feature",
        }
        assert json.loads(result.stdout) == expected

    @pytest.mark.skip(reason="not implemented")
    def test_cli_shapes_compact(self) -> None:
        """Output is compact."""
        result = _run_cli(["shapes", "--compact"], "[106, 193, 9]")
        assert result.returncode == 0
        assert '"type":"Feature"' in result.stdout.strip()

    @pytest.mark.skip(reason="not implemented b/c why would I/anyone ever need that...")
    def test_cli_shapes_indentation(self) -> None:
        """Output is indented."""
        result = _run_cli(["shapes", "--indent", "8"], "[106, 193, 9]")
        assert result.returncode == 0
        assert '        "type": "Feature"' in result.stdout.strip()

    def test_cli_shapes_collect(self) -> None:
        """Shapes are collected into a feature collection."""
        result = _run_cli(["shapes", "--collect", "--feature"], "[106, 193, 9]")
        assert result.returncode == 0
        assert "FeatureCollection" in result.stdout

    def test_cli_shapes_extents(self) -> None:
        result = _run_cli(
            ["shapes", "[106, 193, 9]", "--extents", "--mercator", "--precision", "3"]
        )
        assert result.returncode == 0
        assert result.stdout == "-11740727.545 4852834.052 -11662456.028 4931105.569\n"

    def test_cli_shapes_bbox(self) -> None:
        """JSON text sequences of bboxes are output."""
        result = _run_cli(
            [
                "shapes",
                "[106, 193, 9]",
                "--seq",
                "--bbox",
                "--mercator",
                "--precision",
                "3",
            ],
        )
        assert result.returncode == 0
        assert (
            result.stdout
            == "\x1e\n[-11740727.545,4852834.052,-11662456.028,4931105.569]\n"
        )

    def test_cli_shapes_props_fid(self) -> None:
        result = _run_cli(
            [
                "shapes",
                '{"tile": [106, 193, 9], "properties": {"title": "foo"}, "id": "42"}',
            ],
        )
        assert result.returncode == 0
        assert '"title":"foo"' in result.stdout
        assert '"id":"42"' in result.stdout

    def test_cli_strict_overlap_contain(self) -> None:
        result1 = _run_cli(["shapes"], "[2331,1185,12]")
        assert result1.returncode == 0
        result2 = _run_cli(["tiles", "12"], result1.stdout)
        assert result2.returncode == 0
        assert result2.stdout == "[2331, 1185, 12]\n"
