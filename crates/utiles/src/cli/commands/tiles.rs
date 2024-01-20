use std::io;
use std::io::Write;

use tracing::debug;

use utiles_core::parsing::parse_bbox_ext;
use utiles_core::tiles;
use utiles_core::zoom::ZoomOrZooms;
use utiles_core::{Tile, TileLike};

use crate::cli::args::TilesArgs;
use crate::cli::stdinterator_filter::stdin_filtered;
use crate::gj::parsing::parse_bbox_geojson;

pub enum TileFmt {
    Arr,
    Obj,
    // Tms,
    // Pmtileid,
    // Quadkey,
}

pub trait TileStringFormatter {
    fn format_tile(&self, tile: &Tile) -> String;
}

impl TileStringFormatter for TileFmt {
    fn format_tile(&self, tile: &Tile) -> String {
        match self {
            TileFmt::Arr => tile.json_arr(),
            TileFmt::Obj => tile.json_obj(),
        }
    }
}

pub fn tiles_main(args: TilesArgs, loop_fn: Option<&dyn Fn()>) {
    let lines = stdin_filtered(args.inargs.input);
    let mut stdout = io::stdout();
    let lock = stdout.lock();
    let mut buf = std::io::BufWriter::with_capacity(32 * 1024, lock);
    let tiles = lines
        .map(|l| {
            let s = l.unwrap();
            debug!("l: {:?}", s);
            // try parsing bbox ext first then try geojson
            let t = parse_bbox_ext(&s).or_else(|_| parse_bbox_geojson(&s));
            t.unwrap()
        })
        .flat_map(|b| {
            tiles(
                (b.west, b.south, b.east, b.north),
                ZoomOrZooms::Zoom(args.zoom),
            )
        })
        .enumerate();

    let tile_fmt = if args.fmtopts.obj {
        TileFmt::Obj
    } else {
        TileFmt::Arr
    };

    let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
    for (i, tile) in tiles {
        let tile_str = tile_fmt.format_tile(&tile);
        let out_str = format!("{rs}{tile_str}\n");
        buf.write_all(out_str.as_bytes()).unwrap();
        // writeln!(stdout, "{}{}", rs, tile_fmt.format_tile(&tile)).unwrap();
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
