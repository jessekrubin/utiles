use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use strum_macros::AsRefStr;

use utiles_core::{
    geobbox_merge, parsing::parse_bbox_ext, zoom, BBox, LngLat, ZoomSet, VERSION,
};

use crate::cli::commands::dev::DevArgs;
use crate::cli::commands::serve::ServeArgs;
use crate::cli::commands::shapes::ShapesArgs;
use crate::cli::commands::{analyze_main, header_main, vacuum_main};
use crate::copy::CopyConfig;
use crate::errors::UtilesResult;
use crate::hash_types::HashType;
use crate::mbt::{MbtType, TilesFilter};
use crate::sqlite::InsertStrategy;
use crate::tile_strfmt::TileStringFormatter;

// ██╗   ██╗████████╗██╗██╗     ███████╗███████╗
// ██║   ██║╚══██╔══╝██║██║     ██╔════╝██╔════╝
// ██║   ██║   ██║   ██║██║     █████╗  ███████╗
// ██║   ██║   ██║   ██║██║     ██╔══╝  ╚════██║
// ╚██████╔╝   ██║   ██║███████╗███████╗███████║
//  ╚═════╝    ╚═╝   ╚═╝╚══════╝╚══════╝╚══════╝

/// utiles cli (rust) ~ v{VERSION}
fn about() -> String {
    let is_debug: bool = cfg!(debug_assertions);
    if is_debug {
        format!("utiles cli (rust) ~ v{VERSION} ~ DEBUG")
    } else {
        format!("utiles cli (rust) ~ v{VERSION}")
    }
}

#[derive(Debug, Parser)]
#[command(name = "ut", about = about(), version = VERSION, author, max_term_width = 120)]
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

fn tile_fmt_string_long_help() -> String {
    r#"Format string for tiles (default: `{json_arr}`)

Example:
    > utiles tiles 1 * --fmt "http://thingy.com/{z}/{x}/{y}.png"
    http://thingy.com/1/0/0.png
    http://thingy.com/1/0/1.png
    http://thingy.com/1/1/0.png
    http://thingy.com/1/1/1.png
    > utiles tiles 1 * --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column = {x} AND tile_row = {-y};"
    SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 1;
    SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 0 AND tile_row = 0;
    SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 1;
    SELECT * FROM tiles WHERE zoom_level = 1 AND tile_column = 1 AND tile_row = 0;

fmt-tokens:
    `{json_arr}`/`{json}`  -> [x, y, z]
    `{json_obj}`/`{obj}`   -> {x: x, y: y, z: z}
    `{quadkey}`/`{qk}`     -> quadkey string
    `{pmtileid}`/`{pmid}`  -> pmtile-id
    `{x}`                  -> x tile coord
    `{y}`                  -> y tile coord
    `{z}`                  -> z/zoom level
    `{-y}`/`{yup}`         -> y tile coord flipped/tms
    `{zxy}`                -> z/x/y
    `{bbox}`               -> [w, s, e, n] bbox lnglat (wgs84)
    `{projwin}`            -> ulx,uly,lrx,lry projwin 4 gdal (wgs84)
    `{bbox_web}`           -> [w, s, e, n] bbox web-mercator (epsg:3857)
    `{projwin_web}`        -> ulx,uly,lrx,lry projwin 4 gdal (epsg:3857)
    "#
        .to_string()
}
#[derive(Debug, Parser)]
pub struct TileFmtOptions {
    /// Write tiles as RS-delimited JSON sequence
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub seq: bool,

    /// Format tiles as json objects (equiv to `-F/--fmt "{json_obj}"`)
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub obj: bool,

    /// Format string for tiles (default: `{json_arr}`); see --help for more
    #[arg(
        required = false,
        long,
        alias = "fmt",
        short = 'F',
        conflicts_with = "obj",
        long_help = tile_fmt_string_long_help()
    )]
    pub fmt: Option<String>,
}

impl TileFmtOptions {
    #[must_use]
    pub fn formatter(&self) -> TileStringFormatter {
        if let Some(fmt) = &self.fmt {
            TileStringFormatter::new(fmt)
        } else if self.obj {
            TileStringFormatter::new("{json_obj}")
        } else {
            TileStringFormatter::default()
        }
    }
}
impl From<&TileFmtOptions> for TileStringFormatter {
    fn from(opts: &TileFmtOptions) -> Self {
        opts.formatter()
    }
}

#[derive(Debug, Parser)]
pub struct TilesArgs {
    /// Zoom level (0-30)
    #[arg(required = true, value_parser = clap::value_parser ! (u8).range(0..=30))]
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
pub struct EdgesArgs {
    #[arg(required = false, long, action = clap::ArgAction::SetTrue,)]
    pub wrapx: bool,

    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}
#[derive(Debug, Parser)]
pub struct BurnArgs {
    /// Zoom level (0-30)
    #[arg(required = true, value_parser = clap::value_parser ! (u8).range(0..=30))]
    pub zoom: u8,

    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}
#[derive(Debug, Parser)]
pub struct MergeArgs {
    /// min zoom level (0-30) to merge to
    #[arg(long, short = 'Z', default_value = "0")]
    pub minzoom: u8,

    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub sort: bool,

    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}
#[derive(Debug, Parser)]
pub struct FmtStrArgs {
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
    /// sqlite filepath
    #[arg(required = true)]
    pub filepath: String,

    /// compact/minified json (default: false)
    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub min: bool,
}

#[derive(Debug, Parser)]
pub struct TilesFilterArgs {
    /// bbox(es) (west, south, east, north)
    #[arg(required = false, long, value_parser = parse_bbox_ext, allow_hyphen_values = true)]
    pub bbox: Option<Vec<BBox>>,

    #[command(flatten)]
    pub zoom: Option<ZoomArgGroup>,
}

impl TilesFilterArgs {
    #[must_use]
    pub fn zooms(&self) -> Option<Vec<u8>> {
        match &self.zoom {
            Some(zoom) => zoom.zooms(),
            None => None,
        }
    }

    #[must_use]
    pub fn bboxes(&self) -> Option<Vec<BBox>> {
        self.bbox.clone()
    }

    #[must_use]
    pub fn tiles_filter_maybe(&self) -> Option<TilesFilter> {
        if self.bbox.is_none() && self.zoom.is_none() {
            None
        } else {
            Some(TilesFilter::new(self.bboxes(), self.zooms()))
        }
    }
}

#[derive(Debug, Parser, Clone, clap::ValueEnum)]
pub enum DbtypeOption {
    Flat,
    Hash,
    Norm,
}

impl From<&DbtypeOption> for MbtType {
    fn from(opt: &DbtypeOption) -> Self {
        match opt {
            DbtypeOption::Flat => MbtType::Flat,
            DbtypeOption::Hash => MbtType::Hash,
            DbtypeOption::Norm => MbtType::Norm,
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub struct TouchArgs {
    /// mbtiles filepath
    #[arg(required = true)]
    pub filepath: String,

    /// page size
    #[arg(required = false, long)]
    pub page_size: Option<i64>,

    /// db-type (default: flat)
    #[arg(
        required = false, long = "dbtype", aliases = ["db-type", "mbtype", "mbt-type"], default_value = "flat"
    )]
    pub dbtype: Option<DbtypeOption>,
}

impl TouchArgs {
    #[must_use]
    pub fn mbtype(&self) -> MbtType {
        self.dbtype.as_ref().map_or(MbtType::Flat, |opt| opt.into())
    }
}

#[derive(Debug, Subcommand)]
/// sqlite utils/cmds
pub enum SqliteCommands {
    Analyze(AnalyzeArgs),
    Header(SqliteHeaderArgs),
    Vacuum(VacuumArgs),
}

impl SqliteCommands {
    pub async fn run(&self) -> UtilesResult<()> {
        match self {
            SqliteCommands::Analyze(args) => analyze_main(args).await,
            SqliteCommands::Header(args) => header_main(args).await,
            SqliteCommands::Vacuum(args) => vacuum_main(args).await,
        }
    }
}

// #[derive(Debug, Parser)]
// pub struct SqliteSchemaArgs {
//     #[command(flatten)]
//     pub common: SqliteDbCommonArgs,
// }

#[derive(Debug, Parser)]
/// Analyze sqlite db
pub struct AnalyzeArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = false, long)]
    pub analysis_limit: Option<usize>,
}

#[derive(Debug, Parser)]
/// Dump sqlite db header
pub struct SqliteHeaderArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,
}

#[derive(Debug, Parser)]
/// vacuum sqlite db inplace/into
pub struct VacuumArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// fspath to vacuum db into
    #[arg(required = false)]
    pub into: Option<String>,

    /// Analyze db after vacuum
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub analyze: bool,

    /// page size to set
    #[arg(required = false, long)]
    pub page_size: Option<i64>,
}

#[derive(Debug, Parser)]
pub struct MetadataArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// Output as json object not array
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub obj: bool,

    /// Output as json string for values (default: false)
    #[arg(required = false, long, action = clap::ArgAction::SetTrue, default_value = "false")]
    pub raw: bool,
}

#[derive(Debug, Parser)]
pub struct MetadataSetArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// key or json-fspath
    #[arg(required = true, value_name = "KEY/FSPATH")]
    pub key: String,

    /// value
    #[arg(required = false)]
    pub value: Option<String>,

    /// dryrun (don't actually set)
    #[arg(
        required = false, long, aliases = ["dry-run"], short = 'n', action = clap::ArgAction::SetTrue
    )]
    pub dryrun: bool,
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
    #[arg(
        required = false, long, action = clap::ArgAction::SetTrue,
        default_value = "false", hide = true
    )]
    pub(crate) fix: bool,
}

#[derive(Debug, Parser)]
pub struct InfoArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub(crate) full: bool,

    #[arg(required = false, long, short, visible_aliases = ["stats"], action = clap::ArgAction::SetTrue)]
    pub(crate) statistics: bool,
}

#[derive(Debug, Parser)]
pub struct AggHashArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    #[command(flatten)]
    pub filter_args: TilesFilterArgs,
    // /// bbox(es) (west, south, east, north)
    // #[arg(required = false, long, value_parser = parse_bbox_ext, allow_hyphen_values = true)]
    // pub bbox: Option<Vec<BBox>>,
    /// hash to use for blob-id if copying to normal/hash db type
    #[arg(required = false, long)]
    pub hash: Option<HashType>,
}

#[derive(Debug, Parser)]
pub struct UpdateArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// dryrun (don't actually update)
    #[arg(required = false, long, short = 'n', action = clap::ArgAction::SetTrue)]
    pub(crate) dryrun: bool,
}
#[derive(Debug, Parser)]
pub struct ZxyifyArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// un-zxyify a db
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub(crate) rm: bool,
}
#[derive(Debug, Parser)]
#[command(
    name = "enumerate",
    about = "enumerate db tiles like `tippecanoe-enumerate`"
)]
pub struct EnumerateArgs {
    #[arg(required = true)]
    pub(crate) fspaths: Vec<String>,

    #[command(flatten)]
    pub filter_args: TilesFilterArgs,

    /// tippecanoe-enumerate like output '{relpath} {x} {y} {z}'
    #[arg(required = false, long, short = 't', action = clap::ArgAction::SetTrue)]
    pub(crate) tippecanoe: bool,
}

#[derive(Debug, Parser)]
/// Optimize tiles-db
pub struct OptimizeArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// destination dataset fspath (mbtiles, dirpath)
    #[arg(required = true)]
    pub dst: String,
}

// #[derive(Debug, Parser)]
// pub struct OxipngArgs {
//     #[command(flatten)]
//     pub common: SqliteDbCommonArgs,
//
//     /// destination dataset fspath (mbtiles, dirpath)
//     #[arg(required = true)]
//     pub dst: String,
//
//     /// optimize level
//     #[arg(required = false, long, short, default_value = "2")]
//     pub(crate) opt: u8,
//
//     /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
//     #[arg(required = false, long, short)]
//     pub jobs: Option<u8>,
//
//     /// quiet
//     #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
//     pub(crate) quiet: bool,
// }

#[derive(Debug, Parser)]
pub struct WebpifyArgs {
    #[command(flatten)]
    pub common: SqliteDbCommonArgs,

    /// destination dataset fspath (mbtiles, dirpath)
    #[arg(required = true)]
    pub dst: String,

    /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
    #[arg(required = false, long, short)]
    pub jobs: Option<u8>,

    /// quiet
    #[arg(required = false, long, short, action = clap::ArgAction::SetTrue)]
    pub(crate) quiet: bool,
}

#[derive(Debug, Parser)]
pub struct CommandsArgs {
    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub full: bool,

    #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    pub(crate) table: bool,
    // /// compact/minified json (default: false)
    // #[arg(required = false, short, long, action = clap::ArgAction::SetTrue)]
    // pub min: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    // Alias `aboot` for possible Canadian users as they will not understand
    // `about` -- similarly I often alias `math` to `maths` for british
    // colleagues who would otherwise have no idea what I'm talking about
    /// Echo info about utiles
    #[command(name = "about", visible_alias = "aboot")]
    About,

    /// list all commands
    #[command(name = "commands", visible_alias = "cmds")]
    Commands(CommandsArgs),

    #[command(subcommand, visible_alias = "db")]
    Sqlite(SqliteCommands),

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

    /// Aggregate tile hashes for tiles-db
    #[command(name = "agg-hash")]
    AggHash(AggHashArgs),

    /// Echo metadata (table) as json arr/obj
    #[command(name = "metadata", visible_aliases = ["meta", "md"])]
    Metadata(MetadataArgs),

    /// Set metadata key/value or from `json` file if key is fspath
    #[command(name = "metadata-set", visible_aliases = ["meta-set", "mds"])]
    MetadataSet(MetadataSetArgs),

    /// Update mbtiles db
    #[command(name = "update", visible_aliases = ["up"])]
    Update(UpdateArgs),

    /// Enumerate tiles db
    #[command(name = "enumerate", visible_aliases = ["enum"])]
    Enumerate(EnumerateArgs),

    /// rm-rf dirpath
    #[command(name = "rimraf", visible_alias = "rmrf", hide = true)]
    Rimraf(RimrafArgs),

    /// Echo mbtiles info/stats
    #[command(name = "info")]
    Info(InfoArgs),

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

    /// zxyify/unzxyify tiles-db
    ///
    /// Adds/removes `z/x/y` table/view for querying tiles not inverted
    #[command(name = "zxyify", aliases = ["xyzify"])]
    Zxyify(ZxyifyArgs),

    /*
    ========================================================================
    TILE CLI UTILS - MERCANTILE LIKE CLI
    ========================================================================
    */
    /// Format json-tiles `[x, y, z]` tiles w/ format-string
    ///
    /// fmt-tokens:
    ///     `{json_arr}`/`{json}`  -> [x, y, z]
    ///     `{json_obj}`/`{obj}`   -> {x: x, y: y, z: z}
    ///     `{quadkey}`/`{qk}`     -> quadkey string
    ///     `{pmtileid}`/`{pmid}`  -> pmtile-id
    ///     `{x}`                  -> x tile coord
    ///     `{y}`                  -> y tile coord
    ///     `{z}`                  -> z/zoom level
    ///     `{-y}`/`{yup}`         -> y tile coord flipped/tms
    ///     `{zxy}`                -> z/x/y
    ///
    ///
    /// Example:
    ///     ```
    ///     \> echo "[486, 332, 10]" | utiles fmtstr
    ///     [486, 332, 10]
    ///     \> echo "[486, 332, 10]" | utiles fmtstr --fmt "{x},{y},{z}"
    ///     486,332,10
    ///     \> echo "[486, 332, 10]" | utiles fmt --fmt "SELECT * FROM tiles WHERE zoom_level = {z} AND tile_column = {x} AND tile_row = {y};"
    ///     SELECT * FROM tiles WHERE zoom_level = 10 AND tile_column = 486 AND tile_row = 332;
    ///     ```
    ///
    #[command(
        name = "fmtstr",
        aliases = & ["fmt", "xt"],
        verbatim_doc_comment,
    )]
    Fmt(TileFmtArgs),

    /// Echo the Web Mercator tile at ZOOM level bounding `GeoJSON` [west, south,
    /// east, north] bounding boxes, features, or collections read from stdin.
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Examples:
    ///
    ///   \> echo "[-105.05, 39.95, -105, 40]" | utiles bounding-tile
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
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Examples:
    ///
    ///   \> echo "[486, 332, 10]" | utiles quadkey
    ///   0313102310
    ///   \> echo "0313102310" | utiles quadkey
    ///   [486, 332, 10]
    ///   \> utiles quadkey 0313102310
    ///   [486, 332, 10]
    #[command(name = "quadkey", verbatim_doc_comment, visible_alias = "qk")]
    Quadkey(TileFmtArgs),

    /// Converts tile(s) to/from pmtile-id/[x, y, z]
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Examples:
    ///
    ///   \> echo "[486, 332, 10]" | utiles pmtileid
    ///   506307
    ///   \> echo "506307" | utiles pmtileid
    ///   [486, 332, 10]
    ///   \> utiles pmtileid 506307
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
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Example:
    ///
    ///   \\> echo "[-105.05, 39.95, -105, 40]" | utiles tiles 12
    ///   [852, 1550, 12]
    ///   [852, 1551, 12]
    ///   [853, 1550, 12]
    ///   [853, 1551, 12]
    ///   \> utiles tiles 12 "[-105.05, 39.95, -105, 40]"
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
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    #[command(name = "neighbors")]
    Neighbors(TileFmtArgs),

    /// Echo children tiles of input tiles
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Example:
    ///
    ///   \> echo "[486, 332, 10]" | utiles children
    ///   [972, 664, 11]
    #[command(name = "children", verbatim_doc_comment)]
    Children(ParentChildrenArgs),

    /// Echo parent tile of input tiles
    #[command(name = "parent")]
    Parent(ParentChildrenArgs),

    /// Echo tiles as `GeoJSON` feature collections/sequences
    ///
    /// Input may be a compact newline-delimited sequences of JSON or a
    /// pretty-printed ASCII RS-delimited sequence of JSON (like
    /// <https://tools.ietf.org/html/rfc8142> and
    /// <https://tools.ietf.org/html/rfc7159>).
    ///
    /// Example:
    ///
    ///   \> echo "[486, 332, 10]" | utiles shapes --precision 4 --bbox
    ///   [-9.1406, 53.1204, -8.7891, 53.3309]
    #[command(name = "shapes")]
    Shapes(ShapesArgs),

    /// Burn tiles from `GeoJSON` stream at zoom level (tile coverage)
    #[command(name = "burn")]
    Burn(BurnArgs),

    /// Merge tiles from stream removing parent tiles if children are present
    #[command(name = "merge")]
    Merge(MergeArgs),

    /// Echo edge tiles from stream of xyz tiles
    #[command(name = "edges")]
    Edges(EdgesArgs),

    /// Convert raster mbtiles to webp format
    #[command(
        name = "webpify",
        about = "Convert raster mbtiles to webp format",
        hide = true
    )]
    Webpify(WebpifyArgs),

    /// Convert raster mbtiles to webp format
    #[command(
        name = "optimize",
        about = "Optimize tiles-db",
        aliases = ["opt"],
        hide = true
    )]
    Optimize(OptimizeArgs),

    /// utiles server (wip)
    #[command(name = "serve", hide = true)]
    Serve(ServeArgs),

    /// Development/Playground command (hidden)
    #[command(name = "dev", hide = true)]
    Dev(DevArgs),

    // ========================================================================
    // UNIMPLEMENTED
    // ========================================================================
    /// UNIMPLEMENTED
    #[command(name = "addo", hide = true)]
    Addo,

    /// UNIMPLEMENTED
    #[command(name = "translate", hide = true)]
    Translate,
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
    /// min zoom level (0-30)
    #[arg(long)]
    minzoom: Option<u8>,

    /// max zoom level (0-30)
    #[arg(long)]
    maxzoom: Option<u8>,
}

#[derive(Debug, Parser)]
pub struct ZoomArgGroup {
    /// Zoom level (0-30)
    #[arg(short, long, required = false, value_delimiter = ',', value_parser = zoom::parse_zooms)]
    pub zoom: Option<Vec<Vec<u8>>>,

    /// min zoom level (0-30)
    #[arg(long, conflicts_with = "zoom", aliases = ["min-zoom", "min-z"])]
    pub minzoom: Option<u8>,

    /// max zoom level (0-30)
    #[arg(long, conflicts_with = "zoom", aliases = ["max-zoom", "max-z"])]
    pub maxzoom: Option<u8>,
}

impl ZoomArgGroup {
    #[must_use]
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

#[derive(
    Debug, Copy, Parser, Clone, clap::ValueEnum, strum::EnumString, AsRefStr, Default,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ConflictStrategy {
    #[default]
    Undefined,
    Ignore,
    Replace,
    Abort,
    Fail,
}

impl From<ConflictStrategy> for InsertStrategy {
    fn from(cs: ConflictStrategy) -> Self {
        match cs {
            ConflictStrategy::Undefined => InsertStrategy::None,
            ConflictStrategy::Ignore => InsertStrategy::Ignore,
            ConflictStrategy::Replace => InsertStrategy::Replace,
            ConflictStrategy::Abort => InsertStrategy::Abort,
            ConflictStrategy::Fail => InsertStrategy::Fail,
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

    /// conflict strategy when copying tiles
    #[arg(required = false, long, short, default_value = "undefined")]
    pub conflict: ConflictStrategy,

    /// db-type (default: src type)
    #[arg(required = false, long = "dst-type", aliases = ["dbtype", "dsttype", "mbtype", "mbt-type", "mbtiles-type"])]
    pub dst_type: Option<DbtypeOption>,

    /// hash to use for blob-id if copying to normal/hash db type
    #[arg(required = false, long)]
    pub hash: Option<HashType>,

    /// n-jobs ~ 0=ncpus (default: max(4, ncpus))
    #[arg(required = false, long, short)]
    pub jobs: Option<u8>,

    /// sqlite fast writing mode (default: false) WIP to use streams
    #[arg(required = false, long, hide = true, action = clap::ArgAction::SetTrue)]
    pub stream: bool,
}

impl CopyArgs {
    #[must_use]
    pub fn zooms(&self) -> Option<Vec<u8>> {
        match &self.zoom {
            Some(zoom) => zoom.zooms(),
            None => None,
        }
    }

    #[must_use]
    pub fn zoom_set(&self) -> Option<ZoomSet> {
        self.zooms().map(|zooms| ZoomSet::from_zooms(&zooms))
    }

    #[must_use]
    pub fn bboxes(&self) -> Option<Vec<BBox>> {
        self.bbox.as_ref().map(|bbox| vec![*bbox])
    }

    #[must_use]
    pub fn bounds(&self) -> Option<String> {
        if let Some(bboxes) = self.bboxes() {
            let new_bbox = geobbox_merge(&bboxes);
            Some(new_bbox.mbt_bounds())
        } else {
            None
        }
    }
}

impl From<&CopyArgs> for CopyConfig {
    fn from(args: &CopyArgs) -> CopyConfig {
        let dbtype = args.dst_type.as_ref().map(|dbtype| dbtype.into());
        CopyConfig {
            src: PathBuf::from(&args.src),
            dst: PathBuf::from(&args.dst),
            zset: args.zoom_set(),
            zooms: args.zooms(),
            verbose: true,
            bboxes: args.bboxes(),
            bounds_string: args.bounds(),
            force: false,
            dryrun: false,
            jobs: args.jobs,
            istrat: InsertStrategy::from(args.conflict),
            hash: args.hash,
            dst_type: dbtype,
            stream: args.stream,
        }
    }
}
