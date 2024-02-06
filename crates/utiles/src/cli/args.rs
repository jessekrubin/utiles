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
#[command(name = "ut", about = about(), version = VERSION, long_about = None, author, max_term_width = 88)]
pub struct Cli {
    /// debug mode (print/log a lot of stuff)
    #[arg(long, short, global = true, default_value = "false", help = "debug mode", action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    #[arg(long, global = true, default_value = "false", help = "trace mode", action = clap::ArgAction::SetTrue)]
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
    #[arg(required = true, help = "mbtiles filepath")]
    pub filepath: String,

    /// compact json (default: false)
    #[arg(required = false, short, long, help = "compact json", action = clap::ArgAction::SetTrue)]
    pub min: bool,
}

#[derive(Debug, Parser)]
pub struct TouchArgs {
    #[arg(required = true, help = "mbtiles filepath")]
    pub filepath: String,
}

#[derive(Debug, Parser)]
pub struct MetadataArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = false, long, help = "output as json object not array", action = clap::ArgAction::SetTrue)]
    pub obj: bool,
}

#[derive(Debug, Parser)]
pub struct MetadataSetArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = true, help = "key")]
    pub key: String,

    #[arg(required = true, help = "value")]
    pub value: Option<String>,
}

#[derive(Debug, Parser)]
pub struct TilejsonArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = false, short, long, help = "include tilestats", action = clap::ArgAction::SetTrue)]
    pub tilestats: bool,
}

#[derive(Debug, Parser)]
pub struct LintArgs {
    #[arg(required = true, help = "filepath(s) or dirpath(s)", num_args(1..))]
    pub(crate) fspaths: Vec<String>,

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

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Echo the `tile.json` for mbtiles file
    #[command(name = "tilejson", visible_alias = "tj", alias = "trader-joes", about = "Echo tilejson for mbtiles file(s)", long_about = None)]
    Tilejson(TilejsonArgs),

    /// Create new mbtiles db w/ schema
    #[command(name = "touch", long_about = None)]
    Touch(TouchArgs),

    /// Copy tiles from src -> dst
    #[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None, visible_alias = "cp")]
    Copy(CopyArgs),

    /// Lint mbtiles file(s) (wip)
    #[command(name = "lint", about = "Lint mbtiles file(s)", long_about = None)]
    Lint(LintArgs),

    /// Echo metadata (table) as json arr/obj
    #[command(name = "metadata", visible_aliases = ["meta", "md"], long_about = None)]
    Metadata(MetadataArgs),

    /// Set metadata key/value
    #[command(name = "metadata-set", visible_aliases = ["meta-set", "mds"], long_about = None)]
    MetadataSet(MetadataSetArgs),

    #[command(name = "rimraf", about = "rm-rf dirpath", long_about = None, visible_alias = "rmrf")]
    Rimraf(RimrafArgs),

    /// Echo mbtiles info/stats
    #[command(name = "mbinfo", long_about = None)]
    Mbinfo(MbtilesStatsArgs),

    /// VACUUM sqlite db
    #[command(name = "vacuum", long_about = None, visible_alias = "vac")]
    Vacuum(SqliteDbCommonArgs),

    // #[command(name = "geojsonio", about = "Open mbtiles in geojson.io", long_about = None)]
    // Geojsonio(SqliteDbCommonArgs),
    /// Determine if mbtiles contains a latlong
    #[command(name = "dbcontains", about = "Determine if mbtiles contains a latlong", long_about = None)]
    Contains {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = true, help = "lat/long")]
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
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///
    /// echo "[-105.05, 39.95, -105, 40]" | utiles bounding-tile
    /// [426, 775, 11]
    #[command(
        name = "bounding-tile",
        about = "Echo the bounding tile of a lonlat/bbox/GeoJSON"
    )]
    BoundingTile(TileFmtArgs),

    ///  Converts tiles given as [x, y, z] and/or quadkeys to/from the other format
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Examples:
    ///
    /// echo "[486, 332, 10]" | utiles quadkey
    /// 0313102310
    ///
    /// echo "0313102310" | utiles quadkey
    /// [486, 332, 10]
    #[command(name = "quadkey", visible_alias = "qk", about = "Convert to/from quadkey(s)", long_about = None)]
    Quadkey(TileFmtArgs),

    /// Echos web-mercator tiles at zoom level intersecting given geojson-bbox [west, south,
    /// east, north], geojson-features, or geojson-collections read from stdin.
    ///
    /// Output format is a JSON `[x, y, z]` array by default; use --obj to output a
    /// JSON object `{x: x, y: y, z: z}`.
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///
    /// $ echo "[-105.05, 39.95, -105, 40]" | utiles tiles 12
    /// [852, 1550, 12]
    /// [852, 1551, 12]
    /// [853, 1550, 12]
    /// [853, 1551, 12]
    #[command(name = "tiles", about = "Echo tiles of bbox")]
    Tiles(TilesArgs),

    /// Converts tiles to/from xyz ([x, y, z]) and/or pmtile-id format(s)
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Examples:
    ///
    /// echo "[486, 332, 10]" | utiles pmtileid
    /// 506307
    ///
    /// echo "506307" | utiles pmtileid
    /// [486, 332, 10]
    #[command(name = "pmtileid", visible_alias = "pmid", about = "Convert to/from pmtile id(s)", long_about = None)]
    Pmtileid(TileFmtArgs),

    /// Echo the neighbor tiles for input tiles
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    #[command(name = "neighbors", about = "Echo neighbors of tile(s)", long_about = None)]
    Neighbors(TileFmtArgs),

    /// Echo children tiles of input tiles
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///    echo "[486, 332, 10]" | utiles children
    ///    [972, 664, 11]
    #[command(name = "children", about = "Echo children of tile(s)", long_about = None)]
    Children(ParentChildrenArgs),

    /// Echo parent tile of input tiles
    #[command(name = "parent", about = "Echo parent of tile(s)", long_about = None)]
    Parent(ParentChildrenArgs),

    /// Echo tiles as GeoJSON feature collections/sequences
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a pretty-
    /// printed ASCII RS-delimited sequence of JSON (like
    /// https://tools.ietf.org/html/rfc8142 and
    /// https://tools.ietf.org/html/rfc7159).
    ///
    /// Example:
    ///     echo "[486, 332, 10]" | mercantile shapes --precision 4 --bbox
    ///     [-9.1406, 53.1204, -8.7891, 53.3309]
    #[command(name = "shapes", about = "Echo shapes of tile(s) as GeoJSON", long_about = None)]
    Shapes(ShapesArgs),

    /// utiles server (wip)
    #[command(name = "serve", about = "utiles server (wip)", long_about = None, hide = true)]
    Serve(ServeArgs),

    /// Development/Playground command (hidden)
    #[command(name = "dev", about = "dev command", long_about = None, hide = true, hide = true)]
    Dev(DevArgs),
}

#[derive(Debug, Parser, Clone)]
#[command(name = "rimraf", about = "rm-rf dirpath", long_about = None)]
pub struct RimrafArgs {
    /// dirpath to nuke
    #[arg(required = true, help = "dirpath to rm")]
    pub dirpath: String,

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
                (None, Some(maxzoom)) => {
                    let thingy: Vec<u8> = (0..=maxzoom).collect();
                    println!("thingy: {thingy:?}");
                    Some((0..=maxzoom).collect())
                }
                (None, None) => None,
            },
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None)]
pub struct CopyArgs {
    /// src dataset fspath (mbtiles, dirpath)
    #[arg(required = true, help = "src dataset fspath")]
    pub src: String,

    /// dst dataset fspath (mbtiles, dirpath)
    #[arg(required = true, help = "dst dataset fspath")]
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
