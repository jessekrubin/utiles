use std::path::Path;

use tracing::debug;

use utiles::mbtiles::MbtilesMetadataRow;
use utilesqlite::Mbtiles;

use crate::args::SqliteDbCommonArgs;

pub fn metadata_main(args: SqliteDbCommonArgs) {
    debug!("meta: {}" , args.filepath);
    // check that filepath exists and is file
    let filepath = Path::new(&args.filepath);
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
    // let mbtiles = Mbtiles::from_filepath(&filepath).unwrap();
    let metadata_rows = mbtiles.metadata().unwrap();
    if args.min {
        let s =
            serde_json::to_string::<Vec<MbtilesMetadataRow>>(&metadata_rows)
                .unwrap();
        println!("{s}");
    } else {
        let s = serde_json::to_string_pretty::<Vec<MbtilesMetadataRow>>(
            &metadata_rows,
        )
            .unwrap();
        println!("{s}");
    }
}