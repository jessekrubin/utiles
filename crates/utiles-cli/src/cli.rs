use std::io::{self, Write};
use std::path::Path;

use crate::args::{Cli, Commands};
use crate::lint::lint_main;
use crate::shapes::shapes_main;
use crate::stdinterator::StdInterator;
use clap::Parser;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;
use utiles::parsing::parse_bbox;
use utiles::tilejson::tilejson_stringify;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;
use utiles::{bounding_tile, Tile};
use utilesqlite::mbtiles::Mbtiles;
// #[group(ArgGroup::new("projected").args(&["geographic", "mercator"]).required(false))]

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

pub async fn cli_main(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    // print args
    let argv = match argv {
        Some(argv) => argv,
        None => std::env::args().collect::<Vec<_>>(),
    };
    let args = Cli::parse_from(&argv);
    let filter = if args.debug {
        EnvFilter::new("DEBUG")
    } else {
        EnvFilter::new("WARN")
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
        Commands::Lint {
            fspaths: filepath,
            fix,
        } => {
            if fix {
                warn!("fix not implemented");
            }
            lint_main(filepath, fix);
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
    0
}

// not sure why this is needed... cargo thinks it's unused???
#[allow(dead_code)]
pub fn cli_main_sync(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli_main(argv, loop_fn).await })
}
