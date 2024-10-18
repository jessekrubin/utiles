import json
import os

import pytest

import utiles as ut
from utiles.dev.testing import run_cli as _run_cli

## TODO union is not implemented as of right now...
# def test_union_cli():
#     filename = os.path.join(os.path.dirname(__file__), 'fixtures/union.txt')
#     expectedFilename = os.path.join(os.path.dirname(__file__), 'expected/union.txt')
#     runner = CliRunner()
#     result = runner.invoke(cli, ['union', filename])
#     assert result.exit_code == 0
#     with open(expectedFilename) as ofile:
#         expected = ofile.readlines()
#     # TODO fuzzy test of featurecollection equality
#     assert len(result.output.strip().split("\n")) == len(expected)


def test_edge_cli() -> None:
    filename = os.path.join(os.path.dirname(__file__), "fixtures", "edges.txt")
    expected_filename = os.path.join(os.path.dirname(__file__), "expected", "edges.txt")
    with open(filename) as f:
        contents = f.read()

    result = _run_cli(["edges"], input=contents)
    assert result.exit_code == 0
    with open(expected_filename) as ofile:
        textiles_expected = ofile.read()
    edge_tiles = set(result.parse_tiles())

    expected_edge_tiles = {
        ut.xyz(*json.loads(e)) for e in textiles_expected.split("\n") if e.strip()
    }
    assert edge_tiles == expected_edge_tiles


def test_burn_cli() -> None:
    filename = os.path.join(os.path.dirname(__file__), "fixtures", "shape.geojson")
    expected_filename = os.path.join(
        os.path.dirname(__file__), "expected", "burned.txt"
    )
    with open(filename) as f:
        geojson = f.read()
    result = _run_cli(["burn", "9"], input=geojson)
    assert result.exit_code == 0
    parsed_edges = result.parse_tiles()
    with open(expected_filename) as f:
        textiles_expected = f.read()

    expected_tiles = set(ut.parse_textiles(textiles_expected))
    # make sure all expected are in parsed_edges...
    for edge in parsed_edges:
        assert edge in expected_tiles


def test_burn_tile_center_point_roundtrip() -> None:
    tile = [83885, 202615, 19]
    w, s, e, n = ut.bounds(*tile)

    x = (e - w) / 2 + w
    y = (n - s) / 2 + s

    point_feature = {
        "type": "Feature",
        "properties": {},
        "geometry": {"type": "Point", "coordinates": [x, y]},
    }
    result = _run_cli(["burn", "19"], input=json.dumps(point_feature))
    assert json.loads(result.output) == tile, result.output


def test_burn_tile_center_lines_roundtrip() -> None:
    tiles = list(ut.children([0, 0, 0]))
    bounds = (ut.bounds(*t) for t in tiles)
    coords = (((e - w) / 2 + w, (n - s) / 2 + s) for w, s, e, n in bounds)

    features = {
        "type": "Feature",
        "properties": {},
        "geometry": {"type": "LineString", "coordinates": list(coords)},
    }

    result = _run_cli(["burn", "1"], input=json.dumps(features))

    output_tiles = [json.loads(t) for t in result.output.split("\n") if t]
    assert sorted(output_tiles) == sorted([list(t) for t in tiles])


@pytest.mark.skip()
def test_burn_cli_tile_shape() -> None:
    # tile_geom = {
    #     "bbox": [-122.4755859375, 37.75334401310657, -122.431640625, 37.78808138412046],
    #     "geometry": {
    #         "coordinates": [
    #             [
    #                 [-122.4755859375, 37.75334401310657],
    #                 [-122.4755859375, 37.78808138412046],
    #                 [-122.431640625, 37.78808138412046],
    #                 [-122.431640625, 37.75334401310657],
    #                 [-122.4755859375, 37.75334401310657],
    #             ]
    #         ],
    #         "type": "Polygon",
    #     },
    #     "id": "(1309, 3166, 13)",
    #     "properties": {"title": "XYZ tile (1309, 3166, 13)"},
    #     "type": "Feature",
    # }
    tile_geom = {
        "bbox": [-122.4755859375, 37.75334401310657, -122.431640625, 37.78808138412046],
        "id": "(1309, 3166, 13)",
        "type": "Feature",
        "geometry": {
            "coordinates": [
                [
                    [-122.4755859375, 37.75334401310657],
                    [-122.4755859375, 37.78808138412046],
                    [-122.431640625, 37.78808138412046],
                    [-122.431640625, 37.75334401310657],
                    [-122.4755859375, 37.75334401310657],
                ]
            ],
            "type": "Polygon",
        },
        "properties": {"title": "XYZ tile (1309, 3166, 13)"},
    }
    result = _run_cli(["burn", "13"], input=json.dumps(tile_geom))
    assert result.output == "[1309, 3166, 13]\n"
