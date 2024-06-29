from __future__ import annotations

from typing import (
    Any,
    Collection,
    Iterable,
    Iterator,
    Literal,
    Optional,
    Sequence,
    Set,
    Tuple,
    TypedDict,
    Union,
    overload,
)

__version_lib__: str
__build_profile__: Literal["debug", "release"]
TupleIntInt = Tuple[int, int]
TupleIntIntInt = Tuple[int, int, int]

TILETYPE_GIF: int
TILETYPE_JPG: int
TILETYPE_JSON: int
TILETYPE_PBF: int
TILETYPE_PBFGZ: int
TILETYPE_PNG: int
TILETYPE_UNKNOWN: int
TILETYPE_WEBP: int

class TileDict(TypedDict):
    x: int
    y: int
    z: int

class LngLat:
    lat: float
    lng: float

    @classmethod
    def __init__(cls, lng: float, lat: float) -> None: ...
    @classmethod
    def from_tile(cls, tile: Tile) -> Any: ...
    def members(self) -> tuple[float, float]: ...
    def __eq__(self, other: Any) -> bool: ...
    def __ge__(self, other: Any) -> bool: ...
    def __getitem__(self, index: int) -> float: ...
    def __gt__(self, other: Any) -> bool: ...
    def __le__(self, other: Any) -> bool: ...
    def __len__(self) -> int: ...
    def __lt__(self, other: Any) -> bool: ...
    def __ne__(self, other: Any) -> bool: ...
    def __hash__(self) -> int: ...
    def __iter__(self) -> Iterator[float]: ...

class Bbox:
    right: float
    top: float
    bottom: float
    left: float

    def __init__(
        self, right: float, top: float, left: float, bottom: float
    ) -> None: ...
    @classmethod
    def from_tile(cls, tile: Tile) -> Bbox: ...
    def members(self) -> Tuple[float, float, float, float]: ...
    @overload
    def __getitem__(self, index: int) -> float: ...
    @overload
    def __getitem__(self, index: slice) -> tuple[float, ...]: ...
    def __len__(self) -> int: ...
    def __hash__(self) -> int: ...
    def __iter__(self) -> Iterator[float]: ...
    def tuple(self) -> Tuple[float, float, float, float]: ...
    def __eq__(self, other: Any) -> bool: ...
    def __ge__(self, other: Any) -> bool: ...

class LngLatBbox:
    east: float
    north: float
    south: float
    west: float

    def __init__(
        self, east: float, north: float, west: float, south: float
    ) -> None: ...
    @classmethod
    def from_tile(cls, tile: Tile) -> LngLatBbox: ...
    def members(self) -> Tuple[float, float, float, float]: ...
    @overload
    def __getitem__(self, index: int) -> float: ...
    @overload
    def __getitem__(self, index: slice) -> tuple[float, ...]: ...
    def __len__(self) -> int: ...
    def __hash__(self) -> int: ...
    def __iter__(self) -> Iterator[float]: ...
    def tuple(self) -> Tuple[float, float, float, float]: ...
    def __eq__(self, other: Any) -> bool: ...
    def __ge__(self, other: Any) -> bool: ...

class Tile:
    def __init__(self, x: int, y: int, z: int) -> None: ...
    def bounds(self) -> LngLatBbox: ...
    def children(self, zoom: int = 1) -> list[Tile]: ...
    def flipy(self) -> Tile: ...
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    @property
    def z(self) -> int: ...
    @classmethod
    def from_lnglat_zoom(cls, lng: float, lat: float, zoom: int) -> Tile: ...
    @classmethod
    def from_quadkey(cls, quadkey: str) -> Tile: ...
    @classmethod
    def from_pmtileid(cls, pmtileid: int) -> Tile: ...
    @classmethod
    def from_row_major_id(cls, rmid: int) -> Tile: ...
    def pmtileid(self) -> int: ...
    def parent_pmtileid(self) -> int: ...
    def row_major_id(self) -> Tuple[int, int, int]: ...
    def rmid(self) -> Tuple[int, int, int]: ...
    def json(self, obj: bool) -> str: ...
    def ll(self) -> LngLat: ...
    def lr(self) -> LngLat: ...
    def members(self) -> tuple[int, int, int]: ...
    def neighbors(self) -> list[Tile]: ...
    def parent(self) -> Tile: ...
    def siblings(self) -> list[Tile]: ...
    def ul(self) -> LngLat: ...
    def ur(self) -> LngLat: ...
    def __eq__(self, other: Any) -> bool: ...
    def __ge__(self, other: Tile | tuple[int, int, int]) -> bool: ...
    def qk(self) -> str: ...
    def quadkey(self) -> str: ...
    def fmt_zxy(self, sep: str | None) -> str: ...
    def fmt_zxy_ext(self, ext: str, sep: str | None) -> str: ...
    @overload
    def __getitem__(self, index: int) -> int: ...
    @overload
    def __getitem__(self, index: slice) -> tuple[int, ...]: ...
    def __gt__(self, other: Tile) -> bool: ...
    def __invert__(self) -> Tile: ...
    def __le__(self, other: Tile) -> bool: ...
    def __len__(self) -> int: ...
    def __lt__(self, other: Tile) -> bool: ...
    def __ne__(self, other: Any) -> bool: ...
    def __hash__(self) -> int: ...
    def tuple(self) -> tuple[int, int, int]: ...
    def __iter__(self) -> Iterator[int]: ...
    def valid(self) -> bool: ...
    def asdict(self) -> TileDict: ...
    def feature(
        self,
        fid: str | None = ...,
        props: dict[Any, Any] | None = ...,
        projected: str | None = ...,
        buffer: float | None = ...,
        precision: int | None = ...,
    ) -> Any: ...
    def center(self) -> LngLat: ...

TileLike = Union[Tile, Tuple[int, int, int], int]

def _parse_tile_arg(*args: TileLike) -> Tile: ...
def _xy(
    lng: float, lat: float, truncate: Optional[bool] = None
) -> Tuple[float, float]: ...
def bounding_tile(
    *bbox: Bbox | tuple[float, float] | float, truncate: bool = ...
) -> Tile: ...
def bounds(tile: TileLike) -> LngLatBbox: ...
def children(*tile: TileLike, zoom: int | None = ...) -> list[Tile]: ...
def feature(
    tile: TileLike,
    fid: str | None = ...,
    props: dict[Any, Any] | None = ...,
    projected: str | None = ...,
    buffer: float | None = ...,
    precision: int | None = ...,
) -> Any: ...
def from_tuple(tile: Tuple[int, int, int]) -> Tile: ...
def lnglat(lng: float, lat: float, truncate: Optional[bool] = None) -> LngLat: ...
def minmax(zoom: int) -> tuple[int, int]: ...
def neighbors(*tile: TileLike) -> list[Tile]: ...
def parent(*tile: TileLike, zoom: int | None = ...) -> Tile | None: ...
def parse_tile_arg(*args: TileLike) -> Tile: ...
def quadkey(*tile: TileLike) -> str: ...
def quadkey2xyz(qk: str) -> Tile: ...
def quadkey_to_tile(qk: str) -> Tile: ...
def tile(
    lng: float, lat: float, zoom: int, truncate: Optional[bool] = None
) -> Tile: ...
def tiles(
    west: float,
    south: float,
    east: float,
    north: float,
    zooms: list[int] | tuple[int, ...] | int,
    truncate: bool = ...,
) -> Collection[Tile]: ...
def tiles_list(
    west: float,
    south: float,
    east: float,
    north: float,
    zooms: list[int] | tuple[int, ...] | int,
    truncate: bool = ...,
) -> list[Tile]: ...
def tiles_count(
    west: float,
    south: float,
    east: float,
    north: float,
    zooms: list[int] | tuple[int, ...] | int,
    truncate: bool = ...,
) -> int: ...
def pmtileid2xyz(pmtileid: int) -> Tile: ...
def qk2xyz(qk: str) -> Tile: ...
def tiletype(buf: bytes) -> int: ...
def tiletype2headers(tiletype_int: int) -> list[tuple[str, str]]: ...
def tiletype_str(buf: bytes) -> str: ...
def ul(*tile: TileLike) -> LngLat: ...
def xy(
    lng: float, lat: float, truncate: Optional[bool] = None
) -> Tuple[float, float]: ...
def xy_bounds(*tile: TileLike) -> Bbox: ...
def xyz2quadkey(x: int, y: int, z: int) -> Any: ...
def xyz(x: int, y: int, z: int) -> Tile: ...
def parse_tiles(tilesish: Sequence[TileLike]) -> list[Tile]: ...
def truncate_lnglat(lng: float, lat: float) -> Tuple[float, float]: ...
def simplify(tiles: Sequence[TileLike]) -> Set[Tile]: ...
def coords(obj: Any) -> Iterable[Tuple[float, float]]: ...
def _coords(obj: Any) -> Iterable[Tuple[float, float]]: ...
def geojson_bounds(obj: Any) -> LngLatBbox: ...
def pmtileid(*tile: TileLike) -> int: ...
def from_pmtileid(pmtileid: int) -> Tile: ...
def geotransform2optzoom(
    geotransform: Tuple[float, float, float, float, float, float],
) -> int: ...

# CLI
def ut_cli(args: list[str]) -> None: ...
def fmt_nbytes(nbytes: int) -> str: ...
