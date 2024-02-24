use clap::{Args, Parser, Subcommand};
use utiles_core::bbox::BBox;
use utiles_core::parsing::parse_bbox_ext;
use utiles_core::zoom;
use utiles_core::LngLat;

use crate::cli::commands::dev::DevArgs;
use crate::cli::commands::serve::ServeArgs;
use crate::cli::commands::shapes::ShapesArgs;

use utiles_core::VERSION;

/// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗
/// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝
/// ██║   ██║   ██║   ██║██║     █████╗  ███████╗
/// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║
/// ╚██████╔╝   ██║   ██║███████╗███████╗███████║
///  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝

fn about() -> String {
    format!("utiles cli (rust) ~ v{VERSION}")
}

#[derive(Debug, Parser)]
#[command(name = "ut", about = about(), version = VERSION, author, max_term_width = 88)]
pub struct Cli {
    /// debug mode (print/log more)
    #[arg(long, global = true, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    /// trace mode (print/log EVEN more)
    #[arg(long, global = true, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub trace: bool,

    /// format log as NDJSON
    #[arg(long, global = true, default_value = "false", action = clap::ArgAction::SetTrue)]
    pub log_json: bool,

    /// CLI subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub struct TileInputStreamArgs {
    #[arg(required = false)]
    pub input: Option<String>,
}

#[derive(Debug, Parser)]
pub struct TileFmtOptions {
    /// Write tiles as RS-delimited JSON sequence
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub seq: bool,

    /// Format tiles as json objects
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub obj: bool,
}

#[derive(Debug, Parser)]
pub struct TilesArgs {
    /// Zoom level (0-32)
    #[arg(required = true)]
    pub zoom: u8,

    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}

#[derive(Debug, Parser)]
pub struct TileFmtArgs {
    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}

#[derive(Debug, Parser)]
pub struct ParentChildrenArgs {
    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,

    #[arg(required = false, long, default_value = "1")]
    pub depth: u8,
}

#[derive(Debug, Parser)]
pub struct SqliteDbCommonArgs {
    /// mbtiles filepath
    #[arg(required = true)]
    pub filepath: String,

    /// compact/minified json (default: false)
    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub min: bool,
}

#[derive(Debug, Parser)]
pub struct TouchArgs {
    /// mbtiles filepath
    #[arg(required = true)]
    pub filepath: String,
}

#[derive(Debug, Parser)]
pub struct VacuumArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// fspath to vacuum db into
    #[arg(required = false)]
    pub into: Option<String>,
}

#[derive(Debug, Parser)]
pub struct MetadataArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// Output as json object not array
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub obj: bool,
}

#[derive(Debug, Parser)]
pub struct MetadataSetArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// key
    #[arg(required = true)]
    pub key: String,

    /// value
    #[arg(required = true)]
    pub value: Option<String>,
}

#[derive(Debug, Parser)]
pub struct TilejsonArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// include tilestats
    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub tilestats: bool,
}

#[derive(Debug, Parser)]
pub struct LintArgs {
    /// filepath(s) or dirpath(s)
    #[arg(required = true, num_args(1..))]
    pub(crate) fspaths: Vec<String>,

    /// fix lint errors (NOT IMPLEMENTED)
    #[arg(required = false, long, action = clap::ArgAction::SetTrue, default_value = "false")]
    pub(crate) fix: bool,
}

#[derive(Debug, Parser)]
pub struct MbtilesStatsArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub(crate) full: bool,
}

#[derive(Debug, Parser)]
pub struct UpdateArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// dryrun (don't actually update)
    #[arg(required = false, long, short = 'n', action = clap::ArgAction::SetTrue)]
    pub(crate) dryrun: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Echo the `tile.json` for mbtiles file
    #[command(name = "tilejson", visible_alias = "tj", alias = "trader-joes")]
    Tilejson(TilejsonArgs),

    /// Create new mbtiles db w/ schema
    #[command(name = "touch")]
    Touch(TouchArgs),

    /// Copy tiles from src -> dst
    #[command(name = "copy", visible_alias = "cp")]
    Copy(CopyArgs),

    /// Lint mbtiles file(s) (wip)
    #[command(name = "lint")]
    Lint(LintArgs),

    /// Echo metadata (table) as json arr/obj
    #[command(name = "metadata", visible_aliases = ["meta", "md"])]
    Metadata(MetadataArgs),

    /// Set metadata key/value
    #[command(name = "metadata-set", visible_aliases = ["meta-set", "mds"])]
    MetadataSet(MetadataSetArgs),

    /// Update mbtiles db
    #[command(name = "update")]
    Update(UpdateArgs),

    /// rm-rf dirpath
    #[command(name = "rimraf", visible_alias = "rmrf")]
    Rimraf(RimrafArgs),

    /// Echo mbtiles info/stats
    #[command(name = "mbinfo")]
    Mbinfo(MbtilesStatsArgs),

    /// VACUUM sqlite db
    #[command(name = "vacuum", visible_alias = "vac")]
    Vacuum(VacuumArgs),

    /// Determine if mbtiles contains a latlong
    #[command(name = "dbcontains")]
    Contains {
        /// mbtiles filepath
        #[arg(required = true)]
        filepath: String,

        /// lat/long
        #[arg(required = true)]
        lnglat: LngLat,
    },

    /*
    ========================================================================
    TILE CLI UTILS - MERCANTILE LIKE CLI
    ========================================================================
    */
    /// Echo the Web Mercator tile at ZOOM level bounding GeoJSON [west, south,
    /// east, north] bounding boxes, features, or collections read from stdin.
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Examples:
    ///
    ///   > echo "[-105.05, 39.95, -105, 40]" | utiles bounding-tile
    ///   [426, 775, 11]
    #[command(
        name = "bounding-tile",
        verbatim_doc_comment,
        about = "Echo bounding tile at zoom for bbox / geojson"
    )]
    BoundingTile(TileFmtArgs),

    /// Converts tiles to/from quadkey/[x, y, z]
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Examples:
    ///
    ///   > echo "[486, 332, 10]" | utiles quadkey
    ///   0313102310
    ///   > echo "0313102310" | utiles quadkey
    ///   [486, 332, 10]
    ///   > utiles quadkey 0313102310
    ///   [486, 332, 10]
    #[command(name = "quadkey", verbatim_doc_comment, visible_alias = "qk")]
    Quadkey(TileFmtArgs),

    /// Converts tile(s) to/from pmtile-id/[x, y, z]
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Examples:
    ///
    ///   > echo "[486, 332, 10]" | utiles pmtileid
    ///   506307
    ///   > echo "506307" | utiles pmtileid
    ///   [486, 332, 10]
    ///   > utiles pmtileid 506307
    ///   [486, 332, 10]
    #[command(name = "pmtileid", verbatim_doc_comment, visible_alias = "pmid")]
    Pmtileid(TileFmtArgs),

    /// Echos web-mercator tiles at zoom level intersecting given geojson-bbox [west, south,
    /// east, north], geojson-features, or geojson-collections read from stdin.
    ///
    /// Output format is a JSON `[x, y, z]` array by default; use --obj to output a
    /// JSON object `{x: x, y: y, z: z}`.
    ///
    /// bbox shorthands (case-insensitive):
    ///     "*"  | "world"     => [-180, -85.0511, 180, 85.0511]
    ///     "n"  | "north"     => [-180, 0, 180, 85.0511]
    ///     "s"  | "south"     => [-180, -85.0511, 180, 0]
    ///     "e"  | "east"      => [0, -85.0511, 180, 85.0511]
    ///     "w"  | "west"      => [-180, -85.0511, 0, 85.0511]
    ///     "ne" | "northeast" => [0, 0, 180, 85.0511]
    ///     "se" | "southeast" => [0, -85.0511, 180, 0]
    ///     "nw" | "northwest" => [-180, 0, 0, 85.0511]
    ///     "sw" | "southwest" => [-180, -85.0511, 0, 0]
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///
    ///   > echo "[-105.05, 39.95, -105, 40]" | utiles tiles 12
    ///   [852, 1550, 12]
    ///   [852, 1551, 12]
    ///   [853, 1550, 12]
    ///   [853, 1551, 12]
    ///   > utiles tiles 12 "[-105.05, 39.95, -105, 40]"
    ///   [852, 1550, 12]
    ///   [852, 1551, 12]
    ///   [853, 1550, 12]
    ///   [853, 1551, 12]
    #[command(
        name = "tiles",
        verbatim_doc_comment,
        about = "Echo tiles at zoom intersecting geojson bbox / feature / collection"
    )]
    Tiles(TilesArgs),

    /// Echo the neighbor tiles for input tiles
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    #[command(name = "neighbors")]
    Neighbors(TileFmtArgs),

    /// Echo children tiles of input tiles
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///
    ///   > echo "[486, 332, 10]" | utiles children
    ///   [972, 664, 11]
    #[command(name = "children", verbatim_doc_comment)]
    Children(ParentChildrenArgs),

    /// Echo parent tile of input tiles
    #[command(name = "parent")]
    Parent(ParentChildrenArgs),

    /// Echo tiles as GeoJSON feature collections/sequences
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///
    ///   > echo "[486, 332, 10]" | utiles shapes --precision 4 --bbox
    ///   [-9.1406, 53.1204, -8.7891, 53.3309]
    #[command(name = "shapes")]
    Shapes(ShapesArgs),

    /// utiles server (wip)
    #[command(name = "serve", hide = true)]
    Serve(ServeArgs),

    /// Development/Playground command (hidden)
    #[command(name = "dev", hide = true)]
    Dev(DevArgs),
}

#[derive(Debug, Parser, Clone)]
#[command(name = "rimraf", about = "rm-rf dirpath")]
pub struct RimrafArgs {
    /// dirpath to nuke
    #[arg(required = true)]
    pub dirpath: String,

    /// collect and print file sizes
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub(crate) size: bool,

    /// dryrun (don't actually rm)
    #[arg(required = false, short = 'n', long, action = clap::ArgAction::SetTrue)]
    pub(crate) dryrun: bool,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false, id = "minmaxzoom")]
pub struct MinMaxZoom {
    /// min zoom level (0-32)
    #[arg(long)]
    minzoom: Option<u8>,

    /// max zoom level (0-32)
    #[arg(long)]
    maxzoom: Option<u8>,
}

// #[group(required = false, multiple = false, id = "zooms")]
#[derive(Debug, Parser)]
pub struct ZoomArgGroup {
    /// Zoom level (0-32)
    #[arg(short, long, required = false, value_delimiter = ',', value_parser = zoom::parse_zooms)]
    pub zoom: Option<Vec<Vec<u8>>>,

    /// min zoom level (0-32)
    #[arg(long, conflicts_with = "zoom")]
    pub minzoom: Option<u8>,

    /// max zoom level (0-32)
    #[arg(long, conflicts_with = "zoom")]
    pub maxzoom: Option<u8>,
}

impl ZoomArgGroup {
    pub fn zooms(&self) -> Option<Vec<u8>> {
        match &self.zoom {
            Some(zooms) => Some(zooms.iter().flatten().copied().collect()),
            None => match (self.minzoom, self.maxzoom) {
                (Some(minzoom), Some(maxzoom)) => Some((minzoom..=maxzoom).collect()),
                (Some(minzoom), None) => Some((minzoom..=31).collect()),
                (None, Some(maxzoom)) => Some((0..=maxzoom).collect()),
                (None, None) => None,
            },
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "copy", about = "Copy tiles from src -> dst")]
pub struct CopyArgs {
    /// source dataset fspath (mbtiles, dirpath)
    #[arg(required = true)]
    pub src: String,

    /// destination dataset fspath (mbtiles, dirpath)
    #[arg(required = true)]
    pub dst: String,

    /// dryrun (don't actually copy)
    #[arg(required = false, long, short = 'n', action = clap::ArgAction::SetTrue)]
    pub dryrun: bool,

    /// force overwrite dst
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub force: bool,

    #[command(flatten)]
    pub zoom: Option<ZoomArgGroup>,

    /// bbox (west, south, east, north)
    #[arg(required = false, long, value_parser = parse_bbox_ext, allow_hyphen_values = true)]
    pub bbox: Option<BBox>,
}

impl CopyArgs {
    pub fn zooms(&self) -> Option<Vec<u8>> {
        match &self.zoom {
            Some(zoom) => zoom.zooms(),
            None => None,
        }
    }
}
