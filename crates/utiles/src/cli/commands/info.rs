use std::path::Path;

use crate::cli::args::InfoArgs;
use crate::errors::UtilesResult;
use crate::mbt::MbtilesStats;
use crate::utilesqlite::Mbtiles;

fn mbinfo(filepath: &str) -> UtilesResult<MbtilesStats> {
    let filepath = Path::new(filepath);
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
    let stats = mbtiles.mbt_stats()?;
    Ok(stats)
}

pub fn info_main(args: &InfoArgs) -> UtilesResult<()> {
    let stats = mbinfo(&args.common.filepath)?;
    let str = if args.common.min {
        serde_json::to_string(&stats)
    } else {
        serde_json::to_string_pretty(&stats)
    }?;
    println!("{str}");
    Ok(())
}
