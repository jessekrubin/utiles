use std::io::BufWriter;
use std::{io, io::Write};

use tracing::{debug, error};

use utiles_core::parsing::parse_bbox_ext;
use utiles_core::zoom::ZoomOrZooms;
use utiles_core::{TileStringFormatter, tiles};

use crate::cli::args::TilesArgs;
use crate::cli::stdinterator_filter::stdin_filtered;
use crate::errors::UtilesResult;
use crate::gj::parsing::parse_bbox_geojson;

pub(crate) async fn tiles_main(
    args: TilesArgs,
    loop_fn: Option<&dyn Fn()>,
) -> UtilesResult<()> {
    let lines = stdin_filtered(args.inargs.input);
    let mut stdout = io::stdout();
    let lock = stdout.lock();
    let mut buf = BufWriter::with_capacity(32 * 1024, lock);
    let tiles_iterators = lines
        .map(|l| match l {
            Ok(s) => {
                debug!("l: {:?}", s);
                Ok(s)
            }
            Err(e) => {
                error!("Error reading line: {:?}", e);
                Err(e)
            }
        })
        .flat_map(|s| {
            s.map(|s| {
                parse_bbox_ext(&s)
                    .or_else(|_| parse_bbox_geojson(&s))
                    .map_err(|e| {
                        error!("Error parsing bbox: {:?}", e);
                        e
                    })
            })
        })
        .map(|parse_result| {
            parse_result.map(|b| {
                tiles(
                    (b.west, b.south, b.east, b.north),
                    ZoomOrZooms::Zoom(args.zoom),
                )
            })
        });
    let formatter = TileStringFormatter::from(&args.fmtopts);
    let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
    for tiles_iterator in tiles_iterators {
        let tiles_iterator = tiles_iterator?;
        for (i, tile) in tiles_iterator.enumerate() {
            let tile_str = formatter.fmt_tile(&tile);
            let out_str = format!("{rs}{tile_str}\n");
            buf.write_all(out_str.as_bytes())?;
            // call loop_fn if it's defined every 1000 iterations for signal break
            if i % 1024 == 0 {
                stdout.flush()?;
                tokio::time::sleep(tokio::time::Duration::from_secs(0)).await;
                if let Some(f) = loop_fn {
                    f();
                }
            }
        }
    }
    buf.flush()?;
    stdout.flush()?;
    Ok(())
}
