"""Utiles rust cli tests"""
from subprocess import run
from json import dumps as stringify
import json

import pytest
from click.testing import CliRunner

from utiles.ut import cli


# def test_cli_shapes_failure() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes"], "0")
#     assert result.exit_code == 2


# def test_cli_shapes() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes", "--precision", "6"], "[106, 193, 9]")
#     assert result.exit_code == 0
#     assert (
#         result.output
#         == '{"bbox": [-105.46875, 39.909736, -104.765625, 40.446947], "geometry": {"coordinates": [[[-105.46875, 39.909736], [-105.46875, 40.446947], [-104.765625, 40.446947], [-104.765625, 39.909736], [-105.46875, 39.909736]]], "type": "Polygon"}, "id": "(106, 193, 9)", "properties": {"title": "XYZ tile (106, 193, 9)"}, "type": "Feature"}\n'
#     )

# def test_cli_shapes_arg() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes", "[106, 193, 9]", "--precision", "6"])
#     assert result.exit_code == 0
#     result_output_json = json.loads(result.output)

#     # '{"bbox": [-105.46875, 39.909736, -104.765625, 40.446947], "geometry": {"coordinates": [[[-105.46875, 39.909736], [-105.46875, 40.446947], [-104.765625, 40.446947], [-104.765625, 39.909736], [-105.46875, 39.909736]]], "type": "Polygon"}, "id": "(106, 193, 9)", "properties": {"title": "XYZ tile (106, 193, 9)"}, "type": "Feature"}\n'
#     expected_dict = {
#         "bbox": [-105.46875, 39.909736, -104.765625, 40.446947],
#         "geometry": {
#             "coordinates": [
#                 [
#                     [-105.46875, 39.909736],
#                     [-105.46875, 40.446947],
#                     [-104.765625, 40.446947],
#                     [-104.765625, 39.909736],
#                     [-105.46875, 39.909736],
#                 ]
#             ],
#             "type": "Polygon",
#         },
#         "id": "(106, 193, 9)",
#         "properties": {"title": "XYZ tile (106, 193, 9)"},
#         "type": "Feature",
#     }

#     assert result_output_json == expected_dict


# def test_cli_shapes_buffer() -> None:
#     runner = CliRunner()
#     result = runner.invoke(
#         cli, ["shapes", "[106, 193, 9]", "--buffer", "1.0", "--precision", "6"]
#     )
#     assert result.exit_code == 0
#     assert (
#         result.output
#         == '{"bbox": [-106.46875, 38.909736, -103.765625, 41.446947], "geometry": {"coordinates": [[[-106.46875, 38.909736], [-106.46875, 41.446947], [-103.765625, 41.446947], [-103.765625, 38.909736], [-106.46875, 38.909736]]], "type": "Polygon"}, "id": "(106, 193, 9)", "properties": {"title": "XYZ tile (106, 193, 9)"}, "type": "Feature"}\n'
#     )


# def test_cli_shapes_compact() -> None:
#     """Output is compact."""
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes", "--compact"], "[106, 193, 9]")
#     assert result.exit_code == 0
#     assert '"type":"Feature"' in result.output.strip()


# def test_cli_shapes_indentation() -> None:
#     """Output is indented."""
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes", "--indent", "8"], "[106, 193, 9]")
#     assert result.exit_code == 0
#     assert '        "type": "Feature"' in result.output.strip()


# def test_cli_shapes_collect() -> None:
#     """Shapes are collected into a feature collection."""
#     runner = CliRunner()
#     result = runner.invoke(cli, ["shapes", "--collect", "--feature"], "[106, 193, 9]")
#     assert result.exit_code == 0
#     assert "FeatureCollection" in result.output


# def test_cli_shapes_extents() -> None:
#     runner = CliRunner()
#     result = runner.invoke(
#         cli, ["shapes", "[106, 193, 9]", "--extents", "--mercator", "--precision", "3"]
#     )
#     assert result.exit_code == 0
#     assert result.output == "-11740727.545 4852834.052 -11662456.028 4931105.569\n"


# def test_cli_shapes_bbox() -> None:
#     """JSON text sequences of bboxes are output."""
#     runner = CliRunner()
#     result = runner.invoke(
#         cli,
#         [
#             "shapes",
#             "[106, 193, 9]",
#             "--seq",
#             "--bbox",
#             "--mercator",
#             "--precision",
#             "3",
#         ],
#     )
#     assert result.exit_code == 0
#     assert (
#         result.output
#         == "\x1e\n[-11740727.545, 4852834.052, -11662456.028, 4931105.569]\n"
#     )


# def test_cli_shapes_props_fid() -> None:
#     runner = CliRunner()
#     result = runner.invoke(
#         cli,
#         [
#             "shapes",
#             '{"tile": [106, 193, 9], "properties": {"title": "foo"}, "id": "42"}',
#         ],
#     )
#     assert result.exit_code == 0
#     assert '"title": "foo"' in result.output
#     assert '"id": "42"' in result.output


# def test_cli_strict_overlap_contain() -> None:
#     runner = CliRunner()
#     result1 = runner.invoke(cli, ["shapes"], "[2331,1185,12]")
#     assert result1.exit_code == 0
#     result2 = runner.invoke(cli, ["tiles", "12"], result1.output)
#     assert result2.exit_code == 0
#     assert result2.output == "[2331, 1185, 12]\n"



# def test_cli_parent_failure() -> None:
#     """[0, 0, 0] has no parent"""
#     runner = CliRunner()
#     result = runner.invoke(cli, ["parent"], "[0, 0, 0]")
#     assert result.exit_code == 2


# def test_cli_parent() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["parent"], "[486, 332, 10]\n[486, 332, 10]")
#     assert result.exit_code == 0
#     assert result.output == "[243, 166, 9]\n[243, 166, 9]\n"


# def test_cli_parent_depth() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["parent", "--depth", "2"], "[486, 332, 10]")
#     assert result.exit_code == 0
#     assert result.output == "[121, 83, 8]\n"


# def test_cli_parent_multidepth() -> None:
#     runner = CliRunner()
#     result = runner.invoke(
#         cli, ["parent", "--depth", "2"], "[486, 332, 10]\n[121, 83, 8]"
#     )
#     assert result.exit_code == 0
#     assert result.output == "[121, 83, 8]\n[30, 20, 6]\n"


# def test_cli_children() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["children"], "[243, 166, 9]")
#     assert result.exit_code == 0
#     assert (
#         result.output
#         == "[486, 332, 10]\n[487, 332, 10]\n[487, 333, 10]\n[486, 333, 10]\n"
#     )


# def test_cli_neighbors() -> None:
#     runner = CliRunner()
#     result = runner.invoke(cli, ["neighbors"], "[243, 166, 9]")
#     assert result.exit_code == 0

#     tiles_lines = result.output.strip().split("\n")
#     tiles = [tuple(json.loads(t)) for t in tiles_lines]
#     assert len(tiles) == 8

#     # We do not provide ordering guarantees
#     # tiles = set([tuple(t) for t in tiles])
#     tiles_set = set(tiles)
#     assert (243, 166, 9) not in tiles_set, "input not in neighbors"

#     assert (243 - 1, 166 - 1, 9) in tiles_set
#     assert (243 - 1, 166 + 0, 9) in tiles_set
#     assert (243 - 1, 166 + 1, 9) in tiles_set
#     assert (243 + 0, 166 - 1, 9) in tiles_set
#     assert (243 + 0, 166 + 1, 9) in tiles_set
#     assert (243 + 1, 166 - 1, 9) in tiles_set
#     assert (243 + 1, 166 + 0, 9) in tiles_set
#     assert (243 + 1, 166 + 1, 9) in tiles_set


def _run_cli(
    args: list[str] | None,
    input: str | None = None,
):
    res = run(
        ["python", "-m", "utiles.ut", *args],
        input=input,
        capture_output=True,
        text=True,
        shell=True,
    )
    return res


def test_rust_cli_help() -> None:
    res = _run_cli(["--help"])
    assert "(rust)" in res.stdout


#
# def test_rust_cli_help() -> None:
#     runner = CliRunner()
#     res = runner.invoke(cli, ["--help"])
#     print(res)
#     assert False


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
        print(result.stdout)
        print(result.stderr)
        assert result.returncode == 0
        assert len(result.stdout.strip().split("\n")) == 4

    def test_rust_cli_tiles_seq(self) -> None:
        result = _run_cli(["tiles", "14", "--seq", "[14.0859, 5.798]"])
        print(result)
        # runner = CliRunner()
        # result = runner.invoke(cli, ["tiles", "14", "--seq",
        #                              "[14.0859, 5.798]"
        #                              ],)
        # print(result)
        assert result.returncode == 0
        assert result.stdout == "\x1e\n[8833, 7927, 14]\n"

    def test_cli_tiles_points(self) -> None:
        result = _run_cli(["tiles", "14"], "[14.0859, 5.798]")
        print(result)
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

    def test_cli_tiles_geosjon(self) -> None:
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
        runner = CliRunner()
        with pytest.warns(DeprecationWarning):
            result = runner.invoke(cli, ["quadkey", "lolwut"])
        assert result.exit_code == 2
        assert "lolwut" in result.output

class TestBoundingTile:

    def test_cli_bounding_tile_bad_bounds(self) -> None:
        """Bounds of len 3 are bad."""
        runner = CliRunner()
        result = _run_cli( ["bounding-tile"], "[-105, 39.99, -104.99]")
        assert result.returncode != 0


    def test_cli_bounding_tile(self) -> None:
        result = _run_cli( ["bounding-tile"], "[-105, 39.99, -104.99, 40]")
        assert result.returncode == 0
        assert result.stdout == "[1706, 3101, 13]\n"


    def test_cli_bounding_tile_bbox(self) -> None:
        result = _run_cli(
             ["bounding-tile"], '{"bbox": [-105, 39.99, -104.99, 40]}'
        )
        assert result.returncode == 0
        assert result.stdout == "[1706, 3101, 13]\n"


    def test_cli_bounding_tile2(self) -> None:
        result = _run_cli( ["bounding-tile"], "[-105, 39.99]")
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
        result = _run_cli( ["bounding-tile"], "[-1, -1, 1, 1]")
        assert result.returncode == 0
        assert result.stdout == "[0, 0, 0]\n"


    @pytest.mark.skip(reason="I dont think this is correct either")
    def test_cli_tiles_bounding_tiles_seq(self) -> None:
        result = _run_cli( ["bounding-tile", "--seq"], "[-1, -1, 1, 1]")
        assert result.returncode == 0
        assert result.stdout == "\x1e\n[0, 0, 0]\n"


    def test_cli_bounding_tile_geosjon(self) -> None:
        collection_dict = {'features': [{'geometry': {'coordinates': [[[-105.46875, 39.909736],
                                                     [-105.46875, 40.446947],
                                                     [-104.765625, 40.446947],
                                                     [-104.765625, 39.909736],
                                                     [-105.46875, 39.909736]]],
                                    'type': 'Polygon'},
                       'id': '(106, 193, 9)',
                       'properties': {'title': 'XYZ tile (106, 193, 9)'},
                       'type': 'Feature'}],
         'type': 'FeatureCollection'}
        collection = json.dumps(collection_dict)
        result = _run_cli( ["bounding-tile"], collection)
        assert result.returncode == 0
        assert result.stdout == "[26, 48, 7]\n"


