use utiles_core::{Tile, TileLike};

use crate::cli::args::ParentChildrenArgs;
use crate::cli::stdinterator_filter;

pub fn parent_main(args: ParentChildrenArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let tiles = lines.map(|l| Tile::from_json(&l.unwrap()).unwrap());
    for tile in tiles {
        let nup = i32::from(tile.z) - i32::from(args.depth);
        // error
        assert!(nup >= 0, "depth must be less than or equal to tile zoom");
        let parent = tile.parent(Option::from(args.depth - 1));
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, parent.json_arr());
    }
}

pub fn children_main(args: ParentChildrenArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let tiles = lines.map(|l| Tile::from_json(&l.unwrap()).unwrap());
    for tile in tiles {
        let children = tile.children(Option::from(tile.z + args.depth));
        for child in children {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            println!("{}{}", rs, child.json_arr());
        }
    }
}
