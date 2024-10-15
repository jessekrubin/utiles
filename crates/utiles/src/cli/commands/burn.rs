use crate::cli::args::TileFmtArgs;
use crate::errors::UtilesResult;
use tracing::debug;

pub async fn burn_main(args: TileFmtArgs) -> UtilesResult<()> {
    debug!("BURN TBD");
    debug!("ARGS: {:?}", args);
    // let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    // let mut tiles: Vec<Tile> =  vec![];
    //
    // for line_res in lines {
    //     let line = line_res?;
    //     let tile = Tile::from_json(&line)?;
    //     tiles.insert(tile);
    //
    //     // let neighbors = tile.neighbors();
    //     // for neighbor in neighbors {
    //     //     let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
    //     //     println!("{}{}", rs, neighbor.json_arr());
    //     // }
    // }
    Ok(())
}
