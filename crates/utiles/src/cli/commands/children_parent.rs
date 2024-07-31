use utiles_core::{Tile, TileLike};

use crate::cli::args::ParentChildrenArgs;
use crate::cli::stdinterator_filter;
use crate::errors::UtilesResult;

pub fn parent_main(args: ParentChildrenArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        let lstr = line?.trim_matches(|c| c == '"' || c == '\'').to_string();
        let tile = Tile::from_json(&lstr)?;
        let nup = i32::from(tile.z) - i32::from(args.depth);
        // error
        assert!(nup >= 0, "depth must be less than or equal to tile zoom");
        let parent = tile.parent(Option::from(args.depth - 1));
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, parent.json_arr());
    }
    Ok(())
}

pub fn children_main(args: ParentChildrenArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        let lstr = line?.trim_matches(|c| c == '"' || c == '\'').to_string();
        let tile = Tile::from_json(&lstr)?;
        let children = tile.children(Option::from(tile.z + args.depth));
        for child in children {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            println!("{}{}", rs, child.json_arr());
        }
    }
    Ok(())
}
