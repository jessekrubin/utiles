use clap::{Args, Parser, Subcommand};
use utiles::bbox::BBox;
use utiles::parsing::parse_bbox_ext;
use utiles::zoom;
use utiles::LngLat;

use crate::commands::dev::DevArgs;
use utiles::VERSION;

use crate::commands::shapes::ShapesArgs;

/// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗
/// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝
/// ██║   ██║   ██║   ██║██║     █████╗  ███████╗
/// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║
/// ╚██████╔╝   ██║   ██║███████╗███████╗███████║
///  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝

fn about() -> String {
    let thingy = format!("utiles cli (rust) ~ v{VERSION}");
    let banner = format!(
        "
  ██╗   ██╗████████╗██╗██╗     ███████╗███████╗
  ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝
  ██║   ██║   ██║   ██║██║     █████╗  ███████╗
  ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║
  ╚██████╔╝   ██║   ██║███████╗███████╗███████║
   ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝
"
    );
    format!("{}\n{}", banner, thingy)
}

#[derive(Debug, Parser)]
#[command(name = "ut", about = about(), version = VERSION, long_about = None, author, max_term_width = 88)]
pub struct Cli {
    /// debug mode (print/log a lot of stuff)
    #[arg(long, short, global = true, default_value = "false", help = "debug mode", action = clap::ArgAction::SetTrue)]
    pub debug: bool,

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
    #[arg(required = true, help = "mbtiles filepath")]
    pub filepath: String,

    #[arg(required = false, short, long, help = "compact json", action = clap::ArgAction::SetTrue)]
    pub min: bool,
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
    pub value: String,
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
    #[command(name = "tilejson", visible_alias = "tj", alias = "trader-joes", about = "Echo tilejson for mbtiles file(s)", long_about = None)]
    Tilejson(TilejsonArgs),

    #[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None, visible_alias = "cp")]
    Copy(CopyArgs),

    #[command(name = "lint", about = "Lint mbtiles file(s)", long_about = None)]
    Lint(LintArgs),

    /// metadata
    #[command(name = "metadata", visible_aliases = ["meta", "md"], about = "Echo metadata (table) as json", long_about = None)]
    Metadata(MetadataArgs),

    #[command(name = "metadata-set", visible_aliases = ["meta-set", "mds"], about = "Set metadata key/value", long_about = None)]
    MetadataSet(MetadataSetArgs),

    #[command(name = "rimraf", about = "rm-rf dirpath", long_about = None, visible_alias = "rmrf")]
    Rimraf(RimrafArgs),

    #[command(name = "mbinfo", about = "Echo basic stats on mbtiles file", long_about = None)]
    Mbinfo(MbtilesStatsArgs),

    // #[command(name = "geojsonio", about = "Open mbtiles in geojson.io", long_about = None)]
    // Geojsonio(SqliteDbCommonArgs),
    #[command(name = "dbcontains", about = "Determine if mbtiles contains a latlong", long_about = None)]
    Contains {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = true, help = "lat/long")]
        lnglat: LngLat,
    },

    // ========================================================================
    // TILE CLI UTILS - MERCANTILE LIKE CLI
    // ========================================================================
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

    #[command(name = "quadkey", visible_alias = "qk", about = "Convert to/from quadkey(s)", long_about = None)]
    Quadkey(TileFmtArgs),

    #[command(name = "tiles", about = "Echo tiles of bbox", long_about = None)]
    Tiles(TilesArgs),

    #[command(name = "pmtileid", visible_alias = "pmid", about = "Convert to/from pmtile id(s)", long_about = None)]
    Pmtileid(TileFmtArgs),

    #[command(name = "neighbors", about = "Echo neighbors of tile(s)", long_about = None)]
    Neighbors(TileFmtArgs),

    #[command(name = "children", about = "Echo children of tile(s)", long_about = None)]
    Children(ParentChildrenArgs),

    #[command(name = "parent", about = "Echo parent of tile(s)", long_about = None)]
    Parent(ParentChildrenArgs),

    #[command(name = "shapes", about = "Echo shapes of tile(s) as GeoJSON", long_about = None)]
    Shapes(ShapesArgs),

    /// Development/Playground command (hidden)
    #[command(name = "dev", about = "dev command", long_about = None, hide = true, hide = true)]
    Dev(DevArgs),
}

#[derive(Debug, Parser, Clone)]
#[command(name = "rimraf", about = "rm-rf dirpath", long_about = None)]
pub struct RimrafArgs {
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

fn parse_zooms(s: &str) -> Result<Option<Vec<u8>>, String> {
    // let r = zoom::parse_zooms(s).unwrap();
    // println!("parse_zooms({:?}) -> {:?}", s, r);
    // Some(r)
    match zoom::parse_zooms(s) {
        Ok(r) => Ok(Some(r)),
        Err(e) => Err(format!("{}", e)),
    }
}
// #[group(required = false, multiple = false, id = "zooms")]
#[derive(Debug, Parser)]
pub struct ZoomArgGroup {
    /// Zoom level (0-32)
    #[arg(short, long, required = false, value_delimiter = ',', value_parser = zoom::parse_zooms)]
    pub zoom: Option<Vec<Vec<u8>>>,
    // /// min zoom level (0-32)
    #[arg(long, conflicts_with = "zoom")]
    pub minzoom: Option<u8>,

    /// max zoom level (0-32)
    #[arg(long, conflicts_with = "zoom")]
    pub maxzoom: Option<u8>,
}

impl ZoomArgGroup {
    pub fn zooms(&self) -> Option<Vec<u8>> {
        match &self.zoom {
            Some(zooms) => Some(zooms.iter().flatten().map(|z| *z).collect()),
            None => match (self.minzoom, self.maxzoom) {
                (Some(minzoom), Some(maxzoom)) => Some((minzoom..=maxzoom).collect()),
                (Some(minzoom), None) => Some((minzoom..=32).collect()),
                (None, Some(maxzoom)) => Some((0..=maxzoom).collect()),
                (None, None) => None,
            },
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None)]
pub struct CopyArgs {
    #[arg(required = true, help = "src dataset fspath")]
    pub src: String,

    #[arg(required = true, help = "dst dataset fspath")]
    pub dst: String,

    /// force overwrite dst
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub force: bool,

    /// args...
    #[command(flatten)]
    pub zoom: Option<ZoomArgGroup>,

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
