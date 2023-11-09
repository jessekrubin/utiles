use std::io::{self, Write};
use std::path::Path;

use clap::{Parser, Subcommand, ValueEnum};
use tracing::debug;
use tracing_subscriber::EnvFilter;
use utiles::parsing::parse_bbox;
use utiles::tilejson::tilejson_stringify;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;
use utiles::{bounding_tile, Tile};
use utilesqlite::mbtiles::Mbtiles;

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

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "lint", about = "lint mbtiles file", long_about = None)]
    Lint {
        #[arg(required = true)]
        filepath: String,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        fix: bool,
    },
    #[command(name = "tilejson", visible_alias = "tj", about = "echo tilejson", long_about = None)]
    Tilejson {
        #[arg(required = true, help = "mbtiles filepath")]
        filepath: String,

        #[arg(required = false, short, long, help= "compact json", action = clap::ArgAction::SetTrue)]
        min: bool,
    },


    // ========================================================================
    // TILE CLI UTILS - MERCANTILE LIKE CLI
    // ========================================================================

    #[command(name = "tiles", about = "echo tiles of bbox", long_about = None)]
    Tiles {
        #[arg(required = true)]
        zoom: u8,

        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },

    // ===================
    // NOT IMPLEMENTED YET
    // ===================
    #[command(name = "quadkey", visible_alias = "qk", about = "convert xyz <-> quadkey", long_about = None)]
    Quadkey {
        #[arg(required = false)]
        input: Option<String>,
    },

    #[command(name = "bounding-tile", about = "output tilejson", long_about = None)]
    BoundingTile {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },
    #[command(name = "neighbors", about = "echo neighbors of tile(s)", long_about = None)]
    Neighbors {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,
    },

    #[command(name = "parent", about = "echo parent of tile(s)", long_about = None)]
    Parent {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,

        #[arg(required = false, long, default_value = "1")]
        depth: u8,
    },
    #[command(name = "children", about = "echo children of tile(s)", long_about = None)]
    Children {
        #[arg(required = false)]
        input: Option<String>,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        seq: bool,

        #[arg(required = false, long, default_value = "1")]
        depth: u8,
    },
    #[command(name = "shapes", about = "echo shapes of tiles as geojson", long_about = None)]
    Shapes {
        #[arg(required = true)]
        input: String,

        seq: bool,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

pub fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };
    let args = Cli::parse_from(&argv);
    // level is info by default and debug if --debug is passed
    // let level = if args.debug {
    //     tracing::Level::DEBUG
    // } else {
    //     tracing::Level::WARN
    // };

    // install global collector configured based on RUST_LOG env var.
    // tracing_subscriber::fmt()
    //     .with_max_level(level)
    //     .with_writer(std::io::stderr)
    //     .finish()
    //     .init();
    // Configure the filter

    let filter = if args.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::from_default_env()
    };

    // Install the global collector configured based on the filter.
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
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
            // let thingy = StdInterator::new(quadkey.quadkey).unwrap();
            // for line in thingy {
            //     println!("Line from stdin: `{}`", line.unwrap());
            // }
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty());
            for line in lines {
                // if the line bgins w '[' treat as tile
                // otherwise treat as quadkey
                let lstr = line.unwrap();
                if lstr.starts_with('[') {
                    // treat as tile
                    let tile = Tile::from_json_arr(&lstr);
                    println!("{}", tile.quadkey());
                    // let qk = utiles::xyz2quadkey(t.west, t.south, t.zoom);
                    // println!("{}", qk);
                    // let tile = parse_bbox(&lstr).unwrap();
                    // let qk = utiles::xyz2quadkey(tile.west, tile.south, tile.zoom);
                    // println!("{}", qk);
                } else {
                    // treat as quadkey
                    let qk = lstr;
                    let tile = Tile::from_quadkey(&qk);
                    if tile.is_err() {
                        println!("Invalid quadkey: {qk}");
                    } else {
                        println!("{}", tile.unwrap().json_arr());
                    }

                    // let (x, y, z) = utiles::quadkey2xyz(&qk);
                    // println!("{} {} {}", x, y, z);
                }
            }
        }
        Commands::BoundingTile { input, seq } => {
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .filter(|l| l.as_ref().unwrap() != "\x1e");

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
            //
            //
            //     .flat_map(|b| tiles(
            //         (b.west, b.south, b.east, b.north),
            //         ZoomOrZooms::Zoom(zoom),
            //     )).enumerate();
            // // let bboxes = lines
            // for (i, tile) in tiles {
            //     let rs = if seq { "\x1e\n" } else { "" };
            //     println!("{}{}", rs, tile.json_arr());
            //     // call loop_fn if it's defined every 1000 iterations for signal break
            //     if i % 1024 == 0 {
            //         if let Some(f) = loop_fn {
            //             f();
            //         }
            //     }
            // }
        }
        Commands::Tiles { zoom, input, seq } => {
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .filter(|l| l.as_ref().unwrap() != "\x1e");
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

            // for tile in tiles {
            //     let tstr =   tile.json_arr();
            //     // RS char if seq else ""
            //     let rs = if seq { "\x1e\n" } else { "" };
            //     println!("{}{}", rs, tstr);
            //     // println!("{}", tile.json_arr());
            //
            //     //     call loop_fn if it's defined
            //     niter += 1;
            //
            //     // call fn every 1000 iterations
            //     if niter % 1000 == 0 {
            //         if let Some(f) = loop_fn {
            //             f();
            //         }
            //     }
            // }
            // for line in input_lines
            //     .filter(|l| !l.is_err())
            //     .filter(|l| !l.as_ref().unwrap().is_empty())
            // {
            //     let lstr = line.unwrap();
            //     let thingy = parse_bbox(
            //         &lstr,
            //     ).unwrap();
            //     for tile in tiles(
            //         (thingy.west, thingy.south, thingy.east, thingy.north),
            //         ZoomOrZooms::Zoom(zoom),
            //     ) {
            //         let tstr =   tile.json_arr();
            //         // RS char if seq else ""
            //         let rs = if seq { "\x1e\n" } else { "" };
            //         println!("{}{}", rs, tstr);
            //         // println!("{}", tile.json_arr());
            //
            //         //     call loop_fn if it's defined
            //         niter += 1;
            //
            //         // call fn every 1000 iterations
            //         if niter % 1000 == 0 {
            //             if let Some(f) = loop_fn {
            //                 f();
            //             }
            //         }
            //     }
            // }
        }
        Commands::Neighbors { input, seq } => {
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .filter(|l| l.as_ref().unwrap() != "\x1e");
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
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .filter(|l| l.as_ref().unwrap() != "\x1e");
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
            let input_lines = StdInterator::new(input).unwrap();
            let lines = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .filter(|l| l.as_ref().unwrap() != "\x1e");
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

        _ => {
            println!("NOT IMPLEMENTED YET");
        }
    }
}
