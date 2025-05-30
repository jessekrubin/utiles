use utiles_core::{Tile, TileLike};

use crate::cli::args::EdgesArgs;
use crate::cli::stdinterator_filter;
use crate::edges::find_edges;
use crate::errors::UtilesResult;

pub(crate) async fn edges_main(args: EdgesArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let mut tiles: Vec<Tile> = vec![];

    for line_res in lines {
        let line = line_res?;
        let tile = Tile::from_json(&line)?;
        tiles.push(tile);
    }

    let titer = find_edges(&tiles, args.wrapx)?;
    for tile in titer {
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        safe_println!("{}{}", rs, tile.json_arr());
    }
    Ok(())
}
