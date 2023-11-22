use crate::shapes::ShapesArgs;
use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ut")]
#[command(about = "utiles cli (rust)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    // debug flag
    #[arg(
        long,
        short,
        global = true,
        default_value = "false",
        help = "debug mode"
    )]
    pub debug: bool,
    // #[command(flatten , help="verbosity level (-v, -vv, -vvv, -vvvv)" )]
    // verbose: Verbosity,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct InputAndSequenceArgs {
    /// The remote to clone
    #[arg(required = false)]
    input: Option<String>,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    seq: bool,
}

#[derive(Debug, Parser)] // requires `derive` feature
pub struct TilesArgs {
    /// The remote to clone
    #[arg(required = true)]
    pub zoom: u8,

    // #[command(flatten)]
    // pub shared: InputAndSequenceArgs,
    #[arg(required = false)]
    pub input: Option<String>,

    #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    pub seq: bool,
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
    // {
    //     #[arg(required = true)]
    //     zoom: u8,
    //
    //     #[arg(required = false)]
    //     input: Option<String>,
    //
    //     #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
    //     seq: bool,
    // },
    #[command(name = "quadkey", visible_alias = "qk", about = "Convert to/from quadkey(s)", long_about = None)]
    Quadkey {
        #[arg(required = false)]
        input: Option<String>,
    },

    #[command(name = "pmtileid", visible_alias = "pmid", about = "Convert to/from pmtile id(s)", long_about = None)]
    PMTileID {
        #[arg(required = false)]
        input: Option<String>,
    },

    #[command(name = "bounding-tile", about = "Echo the bounding tile of a lonlat/bbox/GeoJSON", long_about = None)]
    BoundingTile {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },
    #[command(name = "neighbors", about = "Echo neighbors of tile(s)", long_about = None)]
    Neighbors {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },

    #[command(name = "parent", about = "Echo parent of tile(s)", long_about = None)]
    Parent {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,

        #[arg(required = false, long, default_value = "1")]
        depth: u8,
    },
    #[command(name = "children", about = "Echo children of tile(s)", long_about = None)]
    Children {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,

        #[arg(required = false, long, default_value = "1")]
        depth: u8,
    },

    #[command(name = "shapes", about = "Echo shapes of tile(s) as GeoJSON", long_about = None)]
    Shapes(ShapesArgs),

    #[command(name = "copy", about = "Copy tiles from src -> dst", long_about = None)]
    Copy(CopyArgs),

    #[command(name = "dev", about = "dev command", long_about = None, hide = true)]
    Dev {},
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
