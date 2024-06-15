use std::io;
use std::io::Write;

use tracing::debug;

use utiles_core::parsing::parse_bbox_ext;
use utiles_core::tiles;
use utiles_core::zoom::ZoomOrZooms;

use crate::cli::args::TilesArgs;
use crate::cli::stdinterator_filter::stdin_filtered;
use crate::errors::UtilesResult;
use crate::gj::parsing::parse_bbox_geojson;
use crate::tile_strfmt::TileStringFormatter;

pub fn tiles_main(args: TilesArgs, loop_fn: Option<&dyn Fn()>) -> UtilesResult<()> {
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

    let formatter = TileStringFormatter::from(&args.fmtopts);
    let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
    for (i, tile) in tiles {
        let tile_str = formatter.fmt_tile(&tile);
        let out_str = format!("{rs}{tile_str}\n");
        buf.write_all(out_str.as_bytes())?;
        // writeln!(stdout, "{}{}", rs, tile_fmt.format_tile(&tile)).unwrap();
        // call loop_fn if it's defined every 1000 iterations for signal break
        if i % 1024 == 0 {
            stdout.flush()?;
            if let Some(f) = loop_fn {
                f();
            }
        }
    }
    stdout.flush()?;
    Ok(())
}
