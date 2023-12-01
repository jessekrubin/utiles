use clap::{Parser, Subcommand};

use utiles::VERSION;

use crate::commands::shapes::ShapesArgs;

fn about() -> String {
    format!("utiles cli (rust) ~ v{}", VERSION)
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ut", about = about(), version = VERSION, long_about = None, author)]
pub struct Cli {
    /// debug mode (print/log a lot of stuff)
    #[arg(
    long,
    short,
    global = true,
    default_value = "false",
    help = "debug mode",
    action = clap::ArgAction::SetTrue,
    )]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct TileInputStreamArgs {
    #[arg(required = false)]
    pub input: Option<String>,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct TileFmtOptions {
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub seq: bool,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub obj: bool,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct TilesArgs {
    #[arg(required = true)]
    pub zoom: u8,

    #[command(flatten)]
    pub inargs: TileInputStreamArgs,

    #[command(flatten)]
    pub fmtopts: TileFmtOptions,
}

#[derive(Debug, Parser)] // requires `derive` feature
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

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "lint", about = "Lint mbtiles file(s)", long_about = None)]
    Lint {
        #[arg(required = true, help = "filepath(s) or dirpath(s)", num_args(1..))]
        fspaths: Vec<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        fix: bool,
    },
    #[command(name = "tilejson", visible_alias = "tj", about = "Echo tileson for mbtiles file(s)", long_about = None)]
    Tilejson {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = false, short, long, help = "compact json", action = clap::ArgAction::SetTrue)]
        min: bool,

        #[arg(required = false, short, long, help = "include tilestats", action = clap::ArgAction::SetTrue)]
        tilestats: bool,
    },

    #[command(name = "metadata", visible_alias = "md", about = "Echo metadata (table) as json", long_about = None)]
    Meta {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = false, short, long, help = "compact json", action = clap::ArgAction::SetTrue)]
        min: bool,
        // #[arg(required = false, short, long, help= "compact json", action = clap::ArgAction::SetTrue)]
        // raw: bool,
    },

    // ========================================================================
    // TILE CLI UTILS - MERCANTILE LIKE CLI
    // ========================================================================
    #[command(name = "tiles", about = "Echo tiles of bbox", long_about = None)]
    Tiles(TilesArgs),

    #[command(name = "quadkey", visible_alias = "qk", about = "Convert to/from quadkey(s)", long_about = None)]
    Quadkey (TileFmtArgs),
    // Quadkey {
    //     #[arg(required = false)]
    //     input: Option<String>,
    // },

    #[command(name = "pmtileid", visible_alias = "pmid", about = "Convert to/from pmtile id(s)", long_about = None)]
    Pmtileid (TileFmtArgs),

    #[command(name = "bounding-tile", about = "Echo the bounding tile of a lonlat/bbox/GeoJSON", long_about = None)]
    BoundingTile (TileFmtArgs),
    #[command(name = "neighbors", about = "Echo neighbors of tile(s)", long_about = None)]
    Neighbors(TileFmtArgs),

    #[command(name = "parent", about = "Echo parent of tile(s)", long_about = None)]
    Parent(ParentChildrenArgs),
    // Parent {
    //     #[arg(required = false)]
    //     input: Option<String>,
    //
    //     #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    //     seq: bool,
    //
    //     #[arg(required = false, long, default_value = "1")]
    //     depth: u8,
    // },
    #[command(name = "children", about = "Echo children of tile(s)", long_about = None)]
    Children(ParentChildrenArgs),

    #[command(name = "shapes", about = "Echo shapes of tile(s) as GeoJSON", long_about = None)]
    Shapes(ShapesArgs),

    #[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None)]
    Copy(CopyArgs),

    #[command(name = "rimraf", about = "rm-rf dirpath", long_about = None)]
    Rimraf(RimrafArgs),

    #[command(name = "dev", about = "dev command", long_about = None, hide = true)]
    Dev {},
}

#[derive(Debug, Parser, Clone)] // requires `derive` feature
#[command(name = "rimraf", about = "rm-rf dirpath", long_about = None)]
pub struct RimrafArgs {
    #[arg(required = true, help = "dirpath to rm")]
    pub dirpath: String,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub(crate) size: bool,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None)]
pub struct CopyArgs {
    // #[arg(required = true, help = "src mbtiles filepath")]
    // src: String,
    //
    // #[arg(required = true, help = "dst mbtiles filepath")]
    // dst: String,
    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    overwrite: bool,
}
