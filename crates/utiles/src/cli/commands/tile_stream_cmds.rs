use tracing::{debug, error};

use utiles_core::{bounding_tile, Tile, TileLike};

use crate::cli::args::TileFmtArgs;
use crate::cli::stdinterator_filter;
use crate::gj::parsing::parse_bbox_geojson;

pub fn neighbors_main(args: TileFmtArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let tiles = lines.map(|l| Tile::from_json(&l.unwrap()).unwrap());
    for tile in tiles {
        let neighbors = tile.neighbors();
        for neighbor in neighbors {
            let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
            println!("{}{}", rs, neighbor.json_arr());
        }
    }
}

pub fn bounding_tile_main(args: TileFmtArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let bboxes = lines.map(|l| {
        let s = l.unwrap();
        debug!("l: {:?}", s);
        parse_bbox_geojson(&s).unwrap()
    });
    for bbox in bboxes {
        let tile = bounding_tile(bbox, None);
        // let tile = Tile::from_bbox(&bbox, zoom);
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, tile.json_arr());
    }
}

pub fn pmtileid_main(args: TileFmtArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        // if the line bgins w '[' treat as tile
        let lstr = line
            .unwrap() // remove the `"` and `'` chars from the beginning and end of the line
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string();
        if lstr.starts_with('[') {
            // treat as tile
            let tile = Tile::from_json(&lstr).unwrap();
            println!("{}", tile.pmtileid());
        } else {
            // treat as pmtileid
            let pmid: u64 = lstr.parse().unwrap();
            let tile = Tile::from_pmid(pmid);
            if tile.is_err() {
                error!("Invalid pmtileid: {pmid}");
                println!("Invalid pmtileid: {pmid}");
            } else {
                println!("{}", tile.unwrap().json_arr());
            }
        }
    }
}

pub fn quadkey_main(args: TileFmtArgs) {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    for line in lines {
        // if the line begins w/ '['/'{' treat as json-tile
        // otherwise treat as quadkey
        let lstr = line.unwrap();
        let first_char = lstr.chars().next().unwrap();
        match first_char {
            '[' | '{' => {
                // treat as tile
                let tile = Tile::from_json(&lstr).unwrap();
                println!("{}", tile.quadkey());
            }
            _ => {
                // treat as quadkey
                let qk = lstr;
                let tile = Tile::from_quadkey(&qk);
                if tile.is_err() {
                    error!("Invalid quadkey: {qk}");
                    println!("Invalid quadkey: {qk}");
                } else {
                    println!("{}", tile.unwrap().json_arr());
                }
            }
        }
    }
}
