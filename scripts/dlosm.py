"""Download OpenStreetMap tiles for testing"""

import asyncio
from functools import lru_cache
from os import path

import listless as ll
import shellfish as sh
import utiles as ut
from httpx import AsyncClient
from rich.console import Console

console = Console()

OUT_DIR = "osm-tiles"


@lru_cache(maxsize=128)
def mkdirp_lru(path: str) -> None:
    sh.mkdirp(path)


def osm_tile_url(t: ut.Tile) -> str:
    return f"https://tile.openstreetmap.org/{t.z}/{t.x}/{t.y}.png"


def tile_dirpath(t: ut.Tile) -> str:
    return path.join(str(t.z), str(t.x))


def tile_filepath(t: ut.Tile) -> str:
    return path.join(OUT_DIR, tile_dirpath(t), f"{t.y}.png")


async def download_tile(t: ut.Tile, c: AsyncClient) -> None:
    fpath = tile_filepath(t)
    skip = await sh.file_exists_async(fpath)
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
    await sh.wbytes_async(fpath, r.content)


async def main(client: AsyncClient):
    tiles_gen = ut.tiles(-180, -90, 180, 90, list(range(5)))
    total_tiles = len(tiles_gen)
    ndownloaded = 0
    tiles_chunks_gen = ll.chunks(tiles_gen, 16)
    for chunk in tiles_chunks_gen:
        async with asyncio.TaskGroup() as g:
            for tile in chunk:
                g.create_task(download_tile(tile, client))
        ndownloaded += len(chunk)
        console.log(
            f"Downloaded {len(chunk)} tiles ({ndownloaded}/{total_tiles})",
        )

    sh.wjson(
        path.join(OUT_DIR, "metadata.json"),
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
    async with AsyncClient() as client:
        await main(client)


if __name__ == "__main__":
    asyncio.run(_main())
