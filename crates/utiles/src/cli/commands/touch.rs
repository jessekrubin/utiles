use std::path::Path;

use tracing::{debug, info};

use utiles_core::errors::UtilesCoreResult;

use crate::cli::args::TouchArgs;
use crate::utilesqlite::Mbtiles;

pub fn touch_main(args: &TouchArgs) -> UtilesCoreResult<()> {
    let filepath = &args.filepath;
    debug!("touch: {}", filepath);
    // check that filepath does not exist already
    let fpth = Path::new(filepath);
    assert!(!fpth.exists(), "Already exists: {}", fpth.display());
    let mbtiles = Mbtiles::create(filepath, None)?;
    info!("Created mbtiles: {:?}", filepath);
    let filename_no_ext = fpth.file_stem().unwrap().to_str().unwrap();
    mbtiles.metadata_set("name", filename_no_ext).unwrap();
    Ok(())
}
