use utiles_core::{Tile, TileLike};

use crate::cli::args::TileFmtArgs;
use crate::cli::stdinterator_filter;
use crate::edges::find_edges;
use crate::errors::UtilesResult;

pub async fn edges_main(args: TileFmtArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let mut tiles: Vec<Tile> = vec![];

    for line_res in lines {
        let line = line_res?;
        let tile = Tile::from_json(&line)?;
        tiles.push(tile);
    }

    let edge_tiles = find_edges(&tiles)?;
    for tile in edge_tiles {
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, tile.json_arr());
    }
    Ok(())
}
