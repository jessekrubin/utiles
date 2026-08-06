"""Download OpenStreetMap tiles for testing"""

from __future__ import annotations

import asyncio
import typing as t
from collections.abc import Iterable
from functools import lru_cache
from os import path

import ry
import utiles as ut
from rich.console import Console

console = Console()

_T = t.TypeVar("_T")
OUT_DIR = "osm-tiles"

tile_formatter = ut.TileFmts("https://tile.openstreetmap.org/{z}/{x}/{y}.png")


@lru_cache(maxsize=128)
def mkdirp_lru(path: str) -> None:
    ry.mkdirp(path)


def osm_tile_url(t: ut.Tile) -> str:
    return tile_formatter.format(t)


def osm_tile_url_fstring(t: ut.Tile) -> str:
    return f"https://tile.openstreetmap.org/{t.z}/{t.x}/{t.y}.png"


def tile_dirpath(t: ut.Tile) -> str:
    return path.join(str(t.z), str(t.x))


def tile_filepath(t: ut.Tile) -> str:
    return path.join(OUT_DIR, tile_dirpath(t), f"{t.y}.png")


async def download_tile(t: ut.Tile, c: ry.Client) -> None:
    fpath = tile_filepath(t)
    skip = await ry.exists_async(fpath)
    if skip:
        return
    # ensure directory exists
    mkdirp_lru(path.join(OUT_DIR, tile_dirpath(t)))
    # download tile
    r = await c.get(osm_tile_url(t))
    if r.status_code != 200:
        console.log(
            f"Failed to download tile {t} with status code {r.status_code} - {r.text}",
            style="red",
        )
        return
    content = await r.bytes()
    await ry.write_async(fpath, content)


def _chunks(it: Iterable[_T], n: int) -> Iterable[list[_T]]:
    """Yield successive n-sized chunks from it."""
    it = iter(it)
    while True:
        chunk = []
        try:
            for _ in range(n):
                chunk.append(next(it))
        except StopIteration:
            if chunk:
                yield chunk
            break
        yield chunk


async def main(client: ry.Client):
    tiles_gen = ut.tiles(-180, -90, 180, 90, list(range(5)))
    total_tiles = len(tiles_gen)
    ndownloaded = 0
    tiles_chunks_gen = _chunks(tiles_gen, 16)
    for chunk in tiles_chunks_gen:
        async with asyncio.TaskGroup() as g:
            for tile in chunk:
                g.create_task(download_tile(tile, client))
        ndownloaded += len(chunk)
        console.log(
            f"Downloaded {len(chunk)} tiles ({ndownloaded}/{total_tiles})",
        )

    ry.write(
        path.join(OUT_DIR, "metadata.json"),
        ry.JSON.stringify(
            {
                "bounds": "-180,-85.05113,180,85.05113",
                "center": "0,0,2",
                "description": "osm standard png tiles 256",
                "format": "png",
                "maxzoom": 4,
                "minzoom": 0,
                "name": "osm-standard",
                "type": "overlay",
            },
            fmt=True,
        ),
    )
    ut.ut_cli(
        [
            "cp",
            OUT_DIR,
            "osm-standard.z0z4.mbtiles",
        ]
    )
    console.log("BABOOM! DONE!")


async def _main():
    client = ry.Client()
    await main(client)


if __name__ == "__main__":
    asyncio.run(_main())
