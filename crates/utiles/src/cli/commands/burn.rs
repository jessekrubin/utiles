use crate::cli::args::BurnArgs;
use crate::cli::stdinterator_filter;
use crate::cover::geojson2tiles;
use crate::errors::UtilesResult;
use crate::{asleep0, UtilesError};
use geojson::GeoJson;
use utiles_core::TileLike;

pub(crate) async fn burn_main(args: BurnArgs) -> UtilesResult<()> {
    let lines = stdinterator_filter::stdin_filtered(args.inargs.input);
    let mut string = String::new();
    for line_res in lines {
        let line = line_res?;
        string.push_str(&line);
    }
    let geojson_parse_res = string
        .parse::<GeoJson>()
        .map_err(|e| UtilesError::GeojsonError(e.to_string()));
    let geojson = geojson_parse_res?;
    let tiles = geojson2tiles(&geojson, args.zoom, None)?;
    for (i, tile) in tiles.iter().enumerate() {
        let rs = if args.fmtopts.seq { "\x1e\n" } else { "" };
        println!("{}{}", rs, tile.json_arr());
        if i % 2048 == 0 {
            asleep0!();
        }
    }
    Ok(())
}
