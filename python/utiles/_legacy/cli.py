"""Utiles cli"""
from __future__ import annotations

import json
import logging
import sys
from typing import Any, Dict, Iterable, List, Optional, Tuple, Union

import click

import utiles


def configure_logging(verbosity: int) -> None:
    """Configure logging level

    Parameters
    ----------
    verbosity : int
        The number of `-v` options from the command line.

    Returns
    -------
    None
    """
    log_level = max(10, 30 - 10 * verbosity)
    logging.basicConfig(stream=sys.stderr, level=log_level)


logger = logging.getLogger(__name__)

RS = "\x1e"


def normalize_input(input: str) -> List[str]:
    """Normalize file or string input."""
    try:
        src = click.open_file(input).readlines()
    except OSError:
        src = [input]
    return src


def iter_lines(lines: List[str]) -> Iterable[str]:
    """Iterate over lines of input, stripping and skipping."""
    for line in lines:
        line_stripped = line.strip()
        if line_stripped:
            yield line_stripped


# The CLI command group.
@click.group(name="utiles", help="utiles cli (python)")
@click.option("--verbose", "-v", count=True, help="Increase verbosity.")
@click.option("--quiet", "-q", count=True, help="Decrease verbosity.")
@click.version_option(version=utiles.__version__, message="%(version)s")
@click.pass_context
def cli(ctx: click.Context, verbose: int, quiet: int) -> None:
    """Execute the main utiles command"""
    verbosity = verbose - quiet
    configure_logging(verbosity)
    ctx.obj = {}
    ctx.obj["verbosity"] = verbosity


# Commands are below.


# The shapes command.
@cli.command(short_help="Print the shapes of tiles as GeoJSON.")
# This input is either a filename, stdin, or a string.
@click.argument("input", default="-", required=False)
# Coordinate precision option.
@click.option(
    "--precision", type=int, default=None, help="Decimal precision of coordinates."
)
# JSON formatting options.
@click.option(
    "--indent", default=None, type=int, help="Indentation level for JSON output"
)
@click.option(
    "--compact/--no-compact", default=False, help="Use compact separators (',', ':')."
)
# Geographic (default) or Mercator switch.
@click.option(
    "--geographic",
    "projected",
    flag_value="geographic",
    default=True,
    help="Output in geographic coordinates (the default).",
)
@click.option(
    "--mercator",
    "projected",
    flag_value="mercator",
    help="Output in Web Mercator coordinates.",
)
@click.option(
    "--seq",
    is_flag=True,
    default=False,
    help="Write a RS-delimited JSON sequence (default is LF).",
)
# GeoJSON feature (default) or collection switch. Meaningful only
# when --x-json-seq is used.
@click.option(
    "--feature",
    "output_mode",
    flag_value="feature",
    default=True,
    help="Output as sequence of GeoJSON features (the default).",
)
@click.option(
    "--bbox",
    "output_mode",
    flag_value="bbox",
    help="Output as sequence of GeoJSON bbox arrays.",
)
@click.option(
    "--collect",
    is_flag=True,
    default=False,
    help="Output as a GeoJSON feature collections.",
)
# Optionally write out bboxen in a form that goes
# straight into GDAL utilities like gdalwarp.
@click.option(
    "--extents/--no-extents",
    default=False,
    help="Write shape extents as ws-separated strings (default is False).",
)
# Optionally buffer the shapes by shifting the x and y values of each
# vertex by a constant number of decimal degrees or meters (depending
# on whether --geographic or --mercator is in effect).
@click.option(
    "--buffer",
    type=float,
    default=None,
    help="Shift shape x and y values by a constant number",
)
@click.pass_context
def shapes(
    _ctx: click.Context,
    input: str,
    precision: Optional[int] = None,
    indent: Optional[int] = None,
    projected: str = "geographic",
    output_mode: str = "feature",
    buffer: Optional[float] = None,
    compact: bool = False,
    seq: bool = False,
    collect: bool = False,
    extents: bool = False,
) -> None:
    """Print tiles as GeoJSON feature collections or sequences.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Tile descriptions may be either an [x, y, z] array or a JSON
    object of the form

      {"tile": [x, y, z], "properties": {"name": "foo", ...}}

    In the latter case, the properties object will be used to update
    the properties object of the output feature.

        Example:

        \b
        echo "[486, 332, 10]" | utiles shapes --precision 4 --bbox
        [-9.1406, 53.1204, -8.7891, 53.3309]

    """
    dump_kwds: Dict[str, Union[bool, int, Tuple[str, ...]]] = {"sort_keys": True}
    if indent:
        dump_kwds["indent"] = indent
    if compact:
        dump_kwds["separators"] = (",", ":")

    src = normalize_input(input)
    features = []
    col_xs = []
    col_ys = []

    for _i, line in enumerate(iter_lines(src)):
        obj = json.loads(line)
        if isinstance(obj, dict):
            x, y, z = obj["tile"][:3]
            props = obj.get("properties")
            fid = obj.get("id")
        elif isinstance(obj, list):
            x, y, z = obj[:3]
            props = {}
            fid = None
        else:
            msg = f"{obj}"
            raise click.BadParameter(msg, param=input, param_hint="input")  # type: ignore

        feature = utiles.feature(
            (x, y, z),
            fid=fid,
            props=props,
            projected=projected,
            buffer=buffer,
            precision=precision,
        )
        bbox = feature["bbox"]
        w, s, e, n = bbox
        col_xs.extend([w, e])
        col_ys.extend([s, n])

        if collect:
            features.append(feature)
        elif extents:
            click.echo(" ".join(map(str, bbox)))
        else:
            if seq:
                click.echo(RS)
            if output_mode == "bbox":
                click.echo(json.dumps(bbox, **dump_kwds))  # type: ignore
            elif output_mode == "feature":
                click.echo(json.dumps(feature, **dump_kwds))  # type: ignore

    if collect and features:
        bbox = [min(col_xs), min(col_ys), max(col_xs), max(col_ys)]
        click.echo(
            json.dumps(
                {"type": "FeatureCollection", "bbox": bbox, "features": features},
                **dump_kwds,  # type: ignore
            )
        )


# The tiles command.
@cli.command(
    short_help="Print tiles that overlap or contain a lng/lat point, "
    "bounding box, or GeoJSON objects."
)
# Mandatory Mercator zoom level argument.
@click.argument("zoom", type=int, default=-1)
# This input is either a filename, stdin, or a string.
# Has to follow the zoom arg.
@click.argument("input", default="-", required=False)
@click.option(
    "--seq/--lf",
    default=False,
    help="Write a RS-delimited JSON sequence (default is LF).",
)
@click.pass_context
def tiles(
    _ctx: click.Context,
    zoom: int = -1,
    input: str = "-",
    seq: bool = False,
) -> None:
    """Lists Web Mercator tiles at ZOOM level intersecting
    GeoJSON [west, south, east, north] bounding boxen, features, or
    collections read from stdin. Output is a JSON
    [x, y, z] array.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Example:

    \b
    $ echo "[-105.05, 39.95, -105, 40]" | utiles tiles 12
    [852, 1550, 12]
    [852, 1551, 12]
    [853, 1550, 12]
    [853, 1551, 12]

    """
    src = iter(normalize_input(input))
    first_line = next(src)

    # If input is RS-delimited JSON sequence.
    if first_line.startswith(RS):

        def feature_gen() -> Iterable[Dict[str, Any]]:
            buffer = first_line.strip(RS)
            for line in src:
                if line.startswith(RS):
                    if buffer:
                        yield json.loads(buffer)
                    buffer = line.strip(RS)
                else:
                    buffer += line
            else:
                yield json.loads(buffer)

    else:

        def feature_gen() -> Iterable[Dict[str, Any]]:
            yield json.loads(first_line)
            for line in src:
                yield json.loads(line)

    for obj in feature_gen():
        if isinstance(obj, list):
            bbox = obj
            if len(bbox) == 2:
                bbox += bbox
            elif len(bbox) != 4:
                msg = f"{bbox}"
                raise click.BadParameter(msg, param=input, param_hint="input")
        elif isinstance(obj, dict):
            if "bbox" in obj:
                bbox = obj["bbox"]
            else:
                bbox = utiles.geojson_bounds(obj)

        west, south, east, north = bbox
        epsilon = 1.0e-10

        if east != west and north != south:
            # 2D bbox
            # shrink the bounds a small amount so that
            # shapes/tiles round trip.
            west += epsilon
            south += epsilon
            east -= epsilon
            north -= epsilon

        for tile in utiles.tiles(west, south, east, north, [zoom], truncate=False):
            vals = (tile.x, tile.y, zoom)
            output = json.dumps(vals)
            if seq:
                click.echo(RS)
            click.echo(output)


# The bounding-tile command.
@cli.command(
    "bounding-tile",
    short_help="Print the bounding tile of a lng/lat point, "
    "bounding box, or GeoJSON objects.",
)
# This input is either a filename, stdin, or a string.
@click.argument("input", default="-", required=False)
@click.option(
    "--seq/--lf",
    default=False,
    help="Write a RS-delimited JSON sequence (default is LF).",
)
@click.pass_context
def bounding_tile(_ctx: click.Context, input: str, seq: bool = False) -> None:
    """Print the Web Mercator tile at ZOOM level bounding
    GeoJSON [west, south, east, north] bounding boxes, features, or
    collections read from stdin.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Example:

    \b
    echo "[-105.05, 39.95, -105, 40]" | utiles bounding-tile
    [426, 775, 11]

    """
    src = iter(normalize_input(input))
    first_line = next(src)

    # If input is RS-delimited JSON sequence.
    if first_line.startswith(RS):

        def feature_gen() -> Iterable[Dict[str, Any]]:
            buffer = first_line.strip(RS)
            for line in src:
                if line.startswith(RS):
                    if buffer:
                        yield json.loads(buffer)
                    buffer = line.strip(RS)
                else:
                    buffer += line
            else:
                yield json.loads(buffer)

    else:

        def feature_gen() -> Iterable[Dict[str, Any]]:
            yield json.loads(first_line)
            for line in src:
                yield json.loads(line)

    for obj in feature_gen():
        if isinstance(obj, list):
            bbox = obj
            if len(bbox) == 2:
                bbox += bbox
            elif len(bbox) != 4:
                msg = f"{bbox}"
                raise click.BadParameter(msg, param=input, param_hint="input")

        elif isinstance(obj, dict):
            if "bbox" in obj:
                bbox = obj["bbox"]
            else:
                bbox = utiles.geojson_bounds(obj)

        west, south, east, north = bbox
        vals = utiles.bounding_tile(west, south, east, north, truncate=False)

        # print(vals)
        # output = json.dumps(vals)
        output = vals.json(obj=False)

        if seq:
            click.echo(RS)

        click.echo(output)


# The children command.
@cli.command(short_help="Print the children of the tile.")
@click.argument("input", default="-", required=False)
@click.option(
    "--depth",
    type=int,
    default=1,
    help="Number of zoom levels to traverse (default is 1).",
)
@click.pass_context
def children(_ctx: click.Context, input: str, depth: int = 1) -> None:
    """Takes [x, y, z] tiles as input and writes children to stdout
    in the same form.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Example:

    \b
    echo "[486, 332, 10]" | utiles children
    [972, 664, 11]
    [973, 664, 11]
    [973, 665, 11]
    [972, 665, 11]

    """
    src = normalize_input(input)
    for line in iter_lines(src):
        tiles = [json.loads(line)[:3]]
        for _i in range(depth):
            tiles = sum([utiles.children(t) for t in tiles], [])
        for t in tiles:
            click.echo(t.json(obj=False))


# The parent command.
@cli.command(short_help="Print the parent tile.")
@click.argument("input", default="-", required=False)
@click.option(
    "--depth",
    type=int,
    default=1,
    help="Number of zoom levels to traverse (default is 1).",
)
@click.pass_context
def parent(_ctx: click.Context, input: str, depth: int = 1) -> None:
    """Takes [x, y, z] tiles as input and writes parents to stdout
    in the same form.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Example:

    \b
    echo "[486, 332, 10]" | utiles parent
    [243, 166, 9]

    """
    src = normalize_input(input)
    for line in iter_lines(src):
        parsed = json.loads(line)[:3]
        tile = utiles.parse_tile_arg(parsed)
        if tile[2] - depth < 0:
            msg = f"Invalid parent level: {tile[2] - depth}"
            raise click.UsageError(msg)
        for _i in range(depth):
            ptile = utiles.parent(tile)
            if ptile is None:
                msg = f"Invalid parent level: {tile[2] - depth}"
                raise click.UsageError(msg)
            tile = ptile
        output = tile.json(obj=False)
        click.echo(output)


# The neighbors command.
@cli.command(short_help="Print the neighbors of the tile.")
@click.argument("input", default="-", required=False)
@click.pass_context
def neighbors(_ctx: click.Context, input: str) -> None:
    """Takes [x, y, z] tiles as input and writes adjacent
    tiles on the same zoom level to stdout in the same form.

    There are no ordering guarantees for the output tiles.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Example:

    \b
    echo "[486, 332, 10]" | utiles neighbors
    [485, 331, 10]
    [485, 332, 10]
    [485, 333, 10]
    [486, 331, 10]
    [486, 333, 10]
    [487, 331, 10]
    [487, 332, 10]
    [487, 333, 10]

    """
    src = normalize_input(input)
    for line in iter_lines(src):
        tile = json.loads(line)[:3]
        tiles = utiles.neighbors(tile)
        for t in tiles:
            click.echo(t.json(obj=False))


@cli.command(short_help="Convert to/from quadkeys.")
@click.argument("input", default="-", required=False)
@click.pass_context
def quadkey(_ctx: click.Context, input: str) -> None:
    """Takes [x, y, z] tiles or quadkeys as input and writes
    quadkeys or a [x, y, z] tiles to stdout, respectively.

    Input may be a compact newline-delimited sequences of JSON or
    a pretty-printed ASCII RS-delimited sequence of JSON (like
    https://tools.ietf.org/html/rfc8142 and
    https://tools.ietf.org/html/rfc7159).

    Examples:

    \b
    echo "[486, 332, 10]" | utiles quadkey
    0313102310

    \b
    echo "0313102310" | utiles quadkey
    [486, 332, 10]

    """
    src = normalize_input(input)
    try:
        for line in iter_lines(src):
            if line[0] == "[":
                tile = json.loads(line)[:3]
                output = utiles.quadkey(tile)
            else:
                tile = utiles.quadkey_to_tile(line)
                output = tile.json(obj=False)
            click.echo(output)
    except ValueError as ve:
        e = click.BadParameter(
            f"{input}", param=click.Parameter("input", type=str), param_hint="input"
        )
        raise e from ve


if __name__ == "__main__":
    cli()
