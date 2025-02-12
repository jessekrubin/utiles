use utiles_core::{Tile, TileLike};

use crate::cli::args::ParentChildrenArgs;
use crate::cli::stdinterator_filter;
use crate::errors::UtilesResult;

pub(crate) fn parent_main(args: ParentChildrenArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        let lstr = line?.trim_matches(|c| c == '"' || c == '\'').to_string();
        let tile = Tile::from_json(&lstr)?;
        let nup = i32::from(tile.z) - i32::from(args.depth);
        // error
        assert!(nup >= 0, "depth must be less than or equal to tile zoom");
        if let Some(parent) = tile.parent(Option::from(args.depth - 1)) {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            safe_println!("{}{}", rs, parent.json_arr());
        }
    }
    Ok(())
}

pub(crate) fn children_main(args: ParentChildrenArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
    for line in lines {
        let lstr = line?.trim_matches(|c| c == '"' || c == '\'').to_string();
        let tile = Tile::from_json(&lstr)?;
        let tile_zbox = tile.children_zbox(Option::from(args.depth));

        let children = tile_zbox.into_iter().map(Tile::from);
        for child in children {
            safe_println!("{}{}", rs, child.json_arr());
        }
    }
    Ok(())
}
