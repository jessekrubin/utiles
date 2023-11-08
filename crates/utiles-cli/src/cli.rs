use std::error::Error;
use clap::{Parser, Subcommand, ValueEnum};
use tracing::debug;

use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use utiles::parsing::parse_bbox;
use utiles::tilejson::tilejson_stringify;
use utiles::tiles;
use utiles::Tile;
use utiles::zoom::ZoomOrZooms;
use crate::stdinterator::StdInterator;
use utilesqlite::mbtiles::Mbtiles;


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
pub struct QuadkeyArgs {
    /// The remote to clone
    // #[arg(required = false, default_value = "-")]
    // quadkey: MaybeStdin<String>,
    // #[arg(required = false)]
    // quadkey: Option<String>,
    #[arg(required = false)]
    input: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// quadkey
    // Quadkey {
    //     #[arg(required = true)]
    //     quadkey: String,
    // },

    #[command(name = "lint", about = "lint mbtiles file", long_about = None)]
    Lint {
        #[arg(required = true)]
        filepath: String,

        #[arg(required = false, long, action = clap::ArgAction::SetTrue)]
        fix: bool,
    },
    #[command(name = "tilejson", visible_alias = "tj", about = "output tilejson", long_about = None)]
    Tilejson {
        #[arg(required = true)]
        filepath: String,
    },

    // MERCANTILE CLIKE (cli+like)
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
    Quadkey{
        #[arg(required = false)]
        input: Option<String>,
    },

    #[command(name = "bounding-tile", about = "output tilejson", long_about = None)]
    BoundingTile {
        #[arg(required = true)]
        zoom: u8,

        #[arg(required = true)]
        input: String,

        seq: bool,
    },

    #[command(name = "children", about = "print children of tile(s)", long_about = None)]
    Children {
        #[arg(required = true)]
        depth: u8,

        #[arg(required = true)]
        input: String,

        seq: bool,
    },

    #[command(name="neighbors", about="print neighbors of tile(s)", long_about=None)]
    Neighbors {
        #[arg(required = true)]
        input: String,

        seq: bool,
    },

    #[command(name="parent", about="print parent of tile(s)", long_about=None)]
    Parent {
        #[arg(required = true)]
        input: String,

        seq: bool,
    },

    #[command(name="shapes", about="print shapes of tiles as geojson", long_about=None)]
    Shapes {
        #[arg(required = true)]
        input: String,

        seq: bool,
    },


    // /// Clones repos
    // #[command(arg_required_else_help = true)]
    // Clone {
    //     /// The remote to clone
    //     remote: String,
    // },
    // /// Compare two commits
    // Diff {
    //     #[arg(value_name = "COMMIT")]
    //     base: Option<OsString>,
    //     #[arg(value_name = "COMMIT")]
    //     head: Option<OsString>,
    //     #[arg(last = true)]
    //     path: Option<OsString>,
    //     #[arg(
    //     long,
    //     require_equals = true,
    //     value_name = "WHEN",
    //     num_args = 0..=1,
    //     default_value_t = ColorWhen::Auto,
    //     default_missing_value = "always",
    //     value_enum
    //     )]
    //     color: ColorWhen,
    // },
    // /// pushes things
    // #[command(arg_required_else_help = true)]
    // Push {
    //     /// The remote to target
    //     remote: String,
    // },
    // /// adds things
    // #[command(arg_required_else_help = true)]
    // Add {
    //     /// Stuff to add
    //     #[arg(required = true)]
    //     path: Vec<PathBuf>,
    // },

    // Stash(StashArgs),
    // #[command(external_subcommand)]
    // External(Vec<OsString>),
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

pub fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn() -> ()>) {
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
        Commands::Quadkey{
            input
        } => {
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
                        println!("Invalid quadkey: {}", qk);
                    } else {
                        println!("{}", tile.unwrap().json_arr());
                    }

                    // let (x, y, z) = utiles::quadkey2xyz(&qk);
                    // println!("{} {} {}", x, y, z);
                }
            }
        }
        Commands::Tiles { zoom, input, seq } => {
            let input_lines = StdInterator::new(input).unwrap();
            let mut niter = 0;
            let tiles = input_lines
                .filter(|l| !l.is_err())
                .filter(|l| !l.as_ref().unwrap().is_empty())
                .map(|l| parse_bbox(&l.unwrap()).unwrap())
                .flat_map(|b| tiles(
                    (b.west, b.south, b.east, b.north),
                    ZoomOrZooms::Zoom(zoom),
                )).enumerate();
            // let bboxes = lines
            for (i, tile) in tiles {
                let rs = if seq { "\x1e\n" } else { "" };
                println!("{}{}", rs, tile.json_arr());
                // call loop_fn if it's defined every 1000 iterations for signal break
                if i % 1024 == 0 {
                    if let Some(f) = loop_fn {
                        f();
                    }
                }
            }

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

        Commands::Lint { filepath, fix } => {
            println!("linting: {}", filepath);
            println!("NOT IMPLEMENTED YET");
        }

        Commands::Tilejson { filepath } => {
            println!("tilejson: {}", filepath);
            println!("NOT IMPLEMENTED YET");
            let mbtiles = Mbtiles::from_filepath(
                &filepath
            ).unwrap();
            let tj = mbtiles.tilejson().unwrap();

            let s = tilejson_stringify(&tj, None);

            println!("{}", s);

            // println!(
            //     "{}",
            //     serde_json::to_string_pretty(&tj).unwrap()
            // );
        }

        _ => {
            println!("NOT IMPLEMENTED YET");
        }
    }
}
