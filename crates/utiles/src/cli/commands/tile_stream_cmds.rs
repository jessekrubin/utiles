use tracing::error;

use utiles_core::{Tile, TileLike, bounding_tile};

use crate::TileStringFormatter;
use crate::cli::args::TileFmtArgs;
use crate::cli::stdinterator_filter;
use crate::errors::UtilesResult;
use crate::gj::parsing::parse_bbox_geojson;

pub(crate) fn fmtstr_main(args: TileFmtArgs) -> UtilesResult<()> {
    let tile_formatter = TileStringFormatter::from(&args.fmtopts);
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line_res in lines {
        let line = line_res?;
        let tile = Tile::from_json(&line)?;
        let tile_str = tile_formatter.fmt_tile(&tile);
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{rs}{tile_str}");
    }
    Ok(())
}

pub(crate) fn neighbors_main(args: TileFmtArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line_res in lines {
        let line = line_res?;
        let tile = Tile::from_json(&line)?;
        // TODO: add --wrapx flag?
        let neighbors = tile.neighbors(false);
        for neighbor in neighbors {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            println!("{}{}", rs, neighbor.json_arr());
        }
    }
    Ok(())
}

pub(crate) fn bounding_tile_main(args: TileFmtArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line_res in lines {
        let line = line_res?;
        let bbox = parse_bbox_geojson(&line)?;
        let tile = bounding_tile(bbox, None)?;
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, tile.json_arr());
    }
    Ok(())
}

pub(crate) fn pmtileid_main(args: TileFmtArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        // if the line bgins w '[' treat as tile
        let lstr = line?.trim_matches(|c| c == '"' || c == '\'').to_string();
        if lstr.starts_with('[') || lstr.starts_with('{') {
            // treat as tile
            let tile = Tile::from_json(&lstr)?;
            println!("{}", tile.pmtileid());
        } else {
            // treat as pmtileid
            let pmid = lstr.parse::<u64>();
            if let Ok(pmid) = pmid {
                let tile = Tile::from_pmid(pmid);
                println!("{}", tile.json_arr());
            } else {
                error!("Invalid pmtileid: {lstr}");
                println!("Invalid pmtileid: {lstr}");
            }
        }
    }
    Ok(())
}

pub(crate) fn quadkey_main(args: TileFmtArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        // if the line begins w/ '['/'{' treat as json-tile
        // otherwise treat as quadkey
        let lstr = line?
            .trim_matches(|c| c == ' ' || c == '"' || c == '\'')
            .to_string();
        let maybe_first_char = lstr.chars().next();
        if let Some(first_char) = maybe_first_char {
            match first_char {
                '[' | '{' => {
                    // treat as tile
                    let tile = Tile::from_json(&lstr)?;
                    println!("{}", tile.quadkey());
                }
                _ => {
                    // treat as quadkey
                    let qk = lstr;
                    let tile = Tile::from_quadkey(&qk);
                    if let Ok(tile) = tile {
                        println!("{}", tile.json_arr());
                    } else {
                        error!("Invalid quadkey: {qk}");
                        println!("Invalid quadkey: {qk}");
                    }
                }
            }
        }
    }
    Ok(())
}
