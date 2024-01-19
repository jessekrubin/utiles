use std::path::Path;

use tracing::debug;

use crate::utilejson::tilejson_stringify;
use crate::utilesqlite::Mbtiles;

use crate::cli::args::TilejsonArgs;

pub fn tilejson_main(args: &TilejsonArgs) {
    debug!("tilejson: {}", args.common.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.common.filepath);
    assert!(
        filepath.exists(),
        "File does not exist: {}",
        filepath.display()
    );
    assert!(
        filepath.is_file(),
        "Not a file: {filepath}",
        filepath = filepath.display()
    );
    let mbtiles: Mbtiles = Mbtiles::from(filepath);
    let mut tj = mbtiles.tilejson().unwrap();
    if !args.tilestats {
        tj.other.remove("tilestats");
    }
    let s = tilejson_stringify(&tj, Option::from(!args.common.min));
    println!("{s}");
}
