use std::collections::HashSet;
use utiles_core::{simplify, Tile, TileLike};

use crate::cli::args::MergeArgs;
use crate::cli::stdinterator_filter;
use crate::errors::UtilesResult;

pub(crate) async fn merge_main(args: MergeArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let mut tiles: HashSet<Tile> = HashSet::new();
    for line_res in lines {
        let line = line_res?;
        let tile = Tile::from_json(&line)?;
        tiles.insert(tile);
    }
    let tile_formatter = args.fmtopts.formatter();
    let merged_tiles = simplify(&tiles, Some(args.minzoom));
    if args.sort {
        let mut sorted_tiles: Vec<&Tile> = merged_tiles.iter().collect();
        sorted_tiles.sort_by(|a, b| {
            a.z()
                .cmp(&b.z())
                .then_with(|| a.x().cmp(&b.x()))
                .then_with(|| a.y().cmp(&b.y()))
        });
        for tile in sorted_tiles {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            let tile_str = tile_formatter.fmt(tile);
            safe_println!("{rs}{tile_str}");
        }
    } else {
        for tile in merged_tiles {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            let tile_str = tile_formatter.fmt(&tile);
            safe_println!("{rs}{tile_str}");
        }
    }
    Ok(())
}
