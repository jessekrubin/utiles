use std::io::{self, Write};
use std::path::Path;

use clap::{Parser, Subcommand};
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;
use utiles::parsing::parse_bbox;
use utiles::tilejson::tilejson_stringify;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;
use utiles::{bounding_tile, Tile};
use utilesqlite::mbtiles::Mbtiles;

use crate::shapes::{shapes_main, ShapesArgs};
use crate::stdinterator::StdInterator;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ut")]
#[command(about = "utiles cli (rust)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    // debug flag
    #[arg(
        long,
        short,
        global = true,
        default_value = "false",
        help = "debug mode"
    )]
    debug: bool,
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
// #[group(ArgGroup::new("projected").args(&["geographic", "mercator"]).required(false))]

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "lint", about = "Lint mbtiles file(s)", long_about = None)]
    Lint {
        #[arg(required = true)]
        filepath: String,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        fix: bool,
    },
    #[command(name = "tilejson", visible_alias = "tj", about = "Echo tileson for mbtiles file(s)", long_about = None)]
    Tilejson {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = false, short, long, help = "compact json", action = clap::ArgAction::SetTrue)]
        min: bool,
    },

    #[command(name = "meta", about = "Echo metadata (table) as json", long_about = None)]
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
    Tiles {
        #[arg(required = true)]
        zoom: u8,

        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },

    #[command(name = "quadkey", visible_alias = "qk", about = "Convert to/from quadkey(s)", long_about = None)]
    Quadkey {
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
}

fn stdin_filtered(
    input: Option<String>,
) -> Box<dyn Iterator<Item = io::Result<String>>> {
    let input_lines = StdInterator::new(input).unwrap();
    let filtered_lines = input_lines
        .filter(|l| !l.is_err())
        .filter(|l| !l.as_ref().unwrap().is_empty())
        .filter(|l| l.as_ref().unwrap() != "\x1e");

    Box::new(filtered_lines) as _
}

pub fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };
    let args = Cli::parse_from(&argv);
    let filter = if args.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::from_default_env()
    };
    // Install the global collector configured based on the filter.
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .init();

    debug!("args: {:?}", std::env::args().collect::<Vec<_>>());
    debug!("argv: {:?}", argv);

    debug!("args: {:?}", args);

    match args.command {
        Commands::Lint { filepath, fix } => {
            println!("lint (fix -- {fix}): {filepath}");
            // throw not implemented error
            panic!("not implemented (yet)")
        }
        Commands::Meta { filepath, min } => {
            debug!("meta: {filepath}");
            // check that filepath exists and is file
            let filepath = Path::new(&filepath);
            if !filepath.exists() {
                panic!("File does not exist: {}", filepath.display());
            }
            if !filepath.is_file() {
                panic!("Not a file: {filepath}", filepath = filepath.display());
            }
            let mbtiles: Mbtiles = Mbtiles::from(filepath);
            // let mbtiles = Mbtiles::from_filepath(&filepath).unwrap();
            let metadata_rows = mbtiles.metadata().unwrap();
            if min {
                let s =
                    serde_json::to_string::<Vec<MbtilesMetadataRow>>(&metadata_rows)
                        .unwrap();
                println!("{s}");
            } else {
                let s = serde_json::to_string_pretty::<Vec<MbtilesMetadataRow>>(
                    &metadata_rows,
                )
                .unwrap();
                println!("{s}");
            }
        }

        Commands::Tilejson { filepath, min } => {
            debug!("tilejson: {filepath}");
            // check that filepath exists and is file
            let filepath = Path::new(&filepath);
            if !filepath.exists() {
                panic!("File does not exist: {}", filepath.display());
            }
            if !filepath.is_file() {
                panic!("Not a file: {filepath}", filepath = filepath.display());
            }
            let mbtiles: Mbtiles = Mbtiles::from(filepath);
            // let mbtiles = Mbtiles::from_filepath(&filepath).unwrap();
            let tj = mbtiles.tilejson().unwrap();
            let s = tilejson_stringify(&tj, Option::from(!min));
            println!("{s}");
        }

        // mercantile cli like
        Commands::Quadkey { input } => {
            let lines = stdin_filtered(input);
            for line in lines {
                // if the line bgins w '[' treat as tile
                // otherwise treat as quadkey
                let lstr = line.unwrap();
                if lstr.starts_with('[') {
                    // treat as tile
                    let tile = Tile::from_json_arr(&lstr);
                    println!("{}", tile.quadkey());
                } else {
                    // treat as quadkey
                    let qk = lstr;
                    let tile = Tile::from_quadkey(&qk);
                    if tile.is_err() {
                        error!("Invalid quadkey: {qk}");
                        println!("Invalid quadkey: {qk}");
                    } else {
                        println!("{}", tile.unwrap().json_arr());
                    }
                }
            }
        }
        Commands::BoundingTile { input, seq } => {
            let lines = stdin_filtered(input);
            let bboxes = lines.map(|l| {
                let s = l.unwrap();
                debug!("l: {:?}", s);
                parse_bbox(&s).unwrap()
            });
            for bbox in bboxes {
                let tile = bounding_tile(bbox, None);
                // let tile = Tile::from_bbox(&bbox, zoom);
                let rs = if seq { "\x1e\n" } else { "" };
                println!("{}{}", rs, tile.json_arr());
            }
        }
        Commands::Tiles { zoom, input, seq } => {
            let lines = stdin_filtered(input);
            let mut stdout = io::stdout();
            let tiles = lines
                .map(|l| {
                    let s = l.unwrap();
                    debug!("l: {:?}", s);
                    parse_bbox(&s).unwrap()
                })
                .flat_map(|b| {
                    tiles((b.west, b.south, b.east, b.north), ZoomOrZooms::Zoom(zoom))
                })
                .enumerate();
            // let bboxes = lines
            for (i, tile) in tiles {
                let rs = if seq { "\x1e\n" } else { "" };
                // println!("{}{}", rs, tile.json_arr());
                writeln!(stdout, "{}{}", rs, tile.json_arr()).unwrap();
                // call loop_fn if it's defined every 1000 iterations for signal break
                if i % 1024 == 0 {
                    stdout.flush().unwrap();
                    if let Some(f) = loop_fn {
                        f();
                    }
                }
            }
            stdout.flush().unwrap();
        }

        Commands::Neighbors { input, seq } => {
            let lines = stdin_filtered(input);
            let tiles = lines.map(|l| Tile::from_json(&l.unwrap()));
            for tile in tiles {
                let neighbors = tile.neighbors();
                for neighbor in neighbors {
                    let rs = if seq { "\x1e\n" } else { "" };
                    println!("{}{}", rs, neighbor.json_arr());
                }
            }
        }

        Commands::Children { input, seq, depth } => {
            let lines = stdin_filtered(input);
            let tiles = lines.map(|l| Tile::from_json(&l.unwrap()));
            for tile in tiles {
                let children = tile.children(Option::from(tile.z + depth));
                for child in children {
                    let rs = if seq { "\x1e\n" } else { "" };
                    println!("{}{}", rs, child.json_arr());
                }
            }
        }

        Commands::Parent { input, seq, depth } => {
            let lines = stdin_filtered(input);
            let tiles = lines.map(|l| Tile::from_json(&l.unwrap()));
            for tile in tiles {
                let nup = tile.z as i32 - depth as i32;
                if nup < 0 {
                    // error
                    panic!("depth must be less than or equal to tile zoom");
                }
                let parent = tile.parent(Option::from(depth - 1));
                let rs = if seq { "\x1e\n" } else { "" };
                println!("{}{}", rs, parent.json_arr());
            }
        }
        Commands::Shapes(args) => {
            shapes_main(args);
        }
    }
}
