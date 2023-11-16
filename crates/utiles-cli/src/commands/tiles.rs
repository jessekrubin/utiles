use crate::args::TilesArgs;
use crate::stdinterator_filter::stdin_filtered;
use std::io;
use std::io::Write;
use tracing::debug;
use utiles::parsing::parse_bbox;
use utiles::tiles;
use utiles::zoom::ZoomOrZooms;

pub fn tiles_main(args: TilesArgs, loop_fn: Option<&dyn Fn()>) {
    let lines = stdin_filtered(args.input);
    let mut stdout = io::stdout();
    let tiles = lines
        .map(|l| {
            let s = l.unwrap();
            debug!("l: {:?}", s);
            parse_bbox(&s).unwrap()
        })
        .flat_map(|b| {
            tiles(
                (b.west, b.south, b.east, b.north),
                ZoomOrZooms::Zoom(args.zoom),
            )
        })
        .enumerate();
    // let bboxes = lines
    for (i, tile) in tiles {
        let rs = if args.seq { "\x1e\n" } else { "" };
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
