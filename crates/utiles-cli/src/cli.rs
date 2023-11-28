use std::io::{self};
use std::path::Path;

use crate::args::{Cli, Commands};
use crate::commands::copy::copy_main;
use crate::commands::dev::dev_main;
use crate::commands::rimraf::rimraf_main;
use crate::commands::tiles::tiles_main;
use crate::lint::lint_main;
use crate::shapes::shapes_main;
use crate::stdinterator_filter;
use clap::Parser;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;
use utiles::mbtiles::metadata_row::MbtilesMetadataRow;
use utiles::parsing::parse_bbox;
use utiles::tilejson::tilejson_stringify;
use utiles::{bounding_tile, Tile};
use utilesqlite::mbtiles::Mbtiles;
// #[group(ArgGroup::new("projected").args(&["geographic", "mercator"]).required(false))]

#[allow(clippy::unused_async)]
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
            lint_main(&filepath, fix);
        }
        Commands::Meta { filepath, min } => {
            debug!("meta: {filepath}");
            // check that filepath exists and is file
            let filepath = Path::new(&filepath);
            assert!(
                filepath.exists(),
                "File does not exist: {}",
                filepath.display()
            );
            assert!(
                filepath.is_file(),
                "Not a file: {filepath}",
                filepath = filepath.display()
            );
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

        Commands::Tilejson {
            filepath,
            min,
            tilestats,
        } => {
            debug!("tilejson: {filepath}");
            // check that filepath exists and is file
            let filepath = Path::new(&filepath);
            assert!(
                filepath.exists(),
                "File does not exist: {}",
                filepath.display()
            );
            assert!(
                filepath.is_file(),
                "Not a file: {filepath}",
                filepath = filepath.display()
            );
            let mbtiles: Mbtiles = Mbtiles::from(filepath);
            let mut tj = mbtiles.tilejson().unwrap();
            if !tilestats {
                tj.other.remove("tilestats");
            }
            let s = tilejson_stringify(&tj, Option::from(!min));
            println!("{s}");
        }

        // mercantile cli like
        Commands::Quadkey { input } => {
            let lines = stdinterator_filter::stdin_filtered(input);
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

        // Convert between tile id (xyz) and pmtileid
        Commands::PMTileID { input } => {
            let lines = stdinterator_filter::stdin_filtered(input);
            for line in lines {
                // if the line bgins w '[' treat as tile
                let lstr = line.unwrap();
                if lstr.starts_with('[') {
                    // treat as tile
                    let tile = Tile::from_json_arr(&lstr);
                    println!("{}", tile.pmtileid());
                } else {
                    // treat as pmtileid
                    let pmid: u64 = lstr.parse().unwrap();
                    let tile = Tile::from_pmid(pmid);
                    if tile.is_err() {
                        error!("Invalid pmtileid: {pmid}");
                        println!("Invalid pmtileid: {pmid}");
                    } else {
                        println!("{}", tile.unwrap().json_arr());
                    }
                }
            }
        }

        Commands::BoundingTile { input, seq } => {
            let lines = stdinterator_filter::stdin_filtered(input);
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
        Commands::Tiles(args) => tiles_main(args, loop_fn),
        Commands::Neighbors { input, seq } => {
            let lines = stdinterator_filter::stdin_filtered(input);
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
            let lines = stdinterator_filter::stdin_filtered(input);
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
            let lines = stdinterator_filter::stdin_filtered(input);
            let tiles = lines.map(|l| Tile::from_json(&l.unwrap()));
            for tile in tiles {
                let nup = i32::from(tile.z) - i32::from(depth);
                // error
                assert!(nup >= 0, "depth must be less than or equal to tile zoom");
                let parent = tile.parent(Option::from(depth - 1));
                let rs = if seq { "\x1e\n" } else { "" };
                println!("{}{}", rs, parent.json_arr());
            }
        }
        Commands::Shapes(args) => {
            shapes_main(args);
        }
        Commands::Copy(_args) => {
            // copy_main(args);
            warn!("copy not implemented");
            copy_main().await;
        }
        Commands::Dev {} => {
            dev_main().await;
        }

        Commands::Rimraf(args) => {
            rimraf_main(args).await;
        }
    }
    0
}

// not sure why this is needed... cargo thinks it's unused???
#[allow(dead_code)]
#[must_use]
pub fn cli_main_sync(argv: Option<Vec<String>>, loop_fn: Option<&dyn Fn()>) -> u8 {
    let r = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { cli_main(argv, loop_fn).await });
    r
}
