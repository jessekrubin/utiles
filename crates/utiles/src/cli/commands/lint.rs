use std::path::PathBuf;

use futures::{stream, StreamExt};
use tracing::{debug, warn};

use crate::cli::args::LintArgs;
use crate::errors::UtilesResult;
use crate::globster;
use crate::lint::MbtilesLinter;

// pub fn lint_mbtiles_file(mbtiles: &Mbtiles, _fix: bool) -> Vec<UtilesLintError> {
//     let mut errors = Vec::new();
//     // match mbtiles.magic_number() {
//     //     Ok(magic_number) => {
//     //         match magic_number {
//     //             MBTILES_MAGIC_NUMBER => {}
//     //             // zero
//     //             0 => {
//     //                 errors.push(UtilesLintError::MbtMissingMagicNumber);
//     //             }
//     //             _ => {
//     //                 errors.push(UtilesLintError::MbtUnknownMagicNumber(magic_number));
//     //             }
//     //         }
//     //     }
//     //     Err(e) => {
//     //         errors.push(UtilesLintError::Unknown(e.to_string()));
//     //     }
//     // }
//
//     // let mbtiles = mbtiles_result.unwrap();
//     let has_unique_index_on_metadata_name =
//         mbtiles.has_unique_index_on_metadata().unwrap();
//     let metadata_name_is_primary_key =
//         mbtiles.metadata_table_name_is_primary_key().unwrap();
//
//     let rows = mbtiles.metadata().unwrap();
//
//     if has_unique_index_on_metadata_name || metadata_name_is_primary_key {
//         let duplicate_rows = metadata2duplicates(rows.clone());
//         if !duplicate_rows.is_empty() {
//             errors.extend(
//                 duplicate_rows
//                     .keys()
//                     .map(|k| UtilesLintError::DuplicateMetadataKey(k.clone()))
//                     .collect::<Vec<UtilesLintError>>(),
//             );
//         }
//     } else {
//         errors.push(UtilesLintError::MissingUniqueIndex(
//             "metadata.name".to_string(),
//         ));
//     }
//     let map = metadata2map(&rows);
//     let map_errs = lint_metadata_map(&map);
//     if !map_errs.is_empty() {
//         errors.extend(map_errs);
//     }
//     errors
// }

// pub fn lint_filepath(
//     fspath: &Path,
//     fix: bool,
// ) -> UtilesLintResult<Vec<UtilesLintError>> {
//     let Some(fspath_str) = fspath.to_str() else {
//         return Err(UtilesLintError::InvalidPath(
//             fspath.to_str().unwrap().to_string(),
//         ));
//     };
//     // let fspath_str = match fspath.to_str() {
//     //     Some(s) => s,
//     //     None => {
//     //         return Err(UtilesLintError::InvalidPath(
//     //             fspath.to_str().unwrap().to_string(),
//     //         ))
//     //     }
//     // };
//
//     if !fspath_str.ends_with(".mbtiles") {
//         let conn = match squealite::open(fspath_str) {
//             Ok(conn) => conn,
//             Err(e) => {
//                 warn!("Unable to open file: {}", e);
//                 return Err(UtilesLintError::UnableToOpen(fspath_str.to_string()));
//             }
//         };
//
//         match is_mbtiles(&conn) {
//             Ok(false) => return Ok(vec![]),
//             Ok(true) => {
//                 let mbtiles = Mbtiles::from_conn(conn);
//                 return Ok(lint_mbtiles_file(&mbtiles, fix));
//             }
//             Err(e) => {
//                 warn!("Unable to determine if file is mbtiles: {}", e);
//                 return Err(UtilesLintError::NotAMbtilesDb(fspath_str.to_string()));
//             }
//         }
//     }
//
//     match Mbtiles::from_filepath_str(fspath_str) {
//         Ok(mbtiles) => Ok(lint_mbtiles_file(&mbtiles, fix)),
//         Err(e) => {
//             warn!("ERROR: {}", e);
//             Err(UtilesLintError::UnableToOpen(fspath_str.to_string()))
//         }
//     }
// }

// async fn lint_filepath_async(
//     path: &Path,
//     fix: bool,
// ) -> UtilesLintResult<Vec<UtilesLintError>> {
//     let linter = MbtilesLinter::new(path, fix);
//
//     linter.lint().await
// }

async fn lint_filepaths(fspaths: Vec<PathBuf>, fix: bool) {
    // for each concurrent

    stream::iter(fspaths)
        .for_each_concurrent(4, |path| async move {
            let linter = MbtilesLinter::new(&path, fix);
            let lint_results = linter.lint().await;
            match lint_results {
                Ok(r) => {
                    debug!("r: {:?}", r);
                    // print each err....
                    if r.is_empty() {
                        debug!("OK: {}", path.display());
                    } else {
                        warn!("{} - {} errors found", path.display(), r.len());
                        // let agg_err = UtilesLintError::LintErrors(r);
                        let strings = r
                            .iter()
                            .map(|e| e.format_error(&path.display().to_string()))
                            .collect::<Vec<String>>();
                        let joined = strings.join("\n");
                        println!("{joined}");
                        for err in r {
                            warn!("{}", err.to_string());
                        }
                    }
                }
                Err(e) => {
                    warn!("Error: {}", e);
                }
            }
        })
        .await;

    // for path in fspaths {
    //     let r = lint_filepath(&path, fix);
    //     match r {
    //         Ok(r) => {
    //             debug!("r: {:?}", r);
    //             // print each err....
    //             if r.is_empty() {
    //                 info!("No errors found");
    //             } else {
    //                 warn!("{} - {} errors found", path.display(), r.len());
    //
    //                 // let agg_err = UtilesLintError::LintErrors(r);
    //                 for err in r {
    //                     warn!("{}", err.to_string());
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             warn!("Unable to open file: {}", e);
    //             warn!("Error: {}", e);
    //         }
    //     }
    // }

    // ============
    // SYNC VERSION
    // ============
    // for path in fspaths {
    //     let linter = mbtileslinter::new(&path, fix);
    //     let lint_results = linter.lint().await;
    //     match lint_results {
    //         ok(r) => {
    //             debug!("r: {:?}", r);
    //             // print each err....
    //             if r.is_empty() {
    //                 debug!("{} - ok", path.display());
    //             } else {
    //                 warn!("{} - {} errors found", path.display(), r.len());
    //
    //                 // let agg_err = utileslinterror::linterrors(r);
    //                 for err in r {
    //                     warn!("{}", err.to_string());
    //                 }
    //             }
    //         }
    //         err(e) => {
    //             warn!("unable to open file: {}", e);
    //             warn!("error: {}", e);
    //         }
    //     }
    // }
}

pub async fn lint_main(args: &LintArgs) -> UtilesResult<()> {
    let filepaths = globster::find_filepaths(&args.fspaths)?;
    if args.fix {
        warn!("NOT IMPLEMENTED: `utiles lint --fix`");
    }
    debug!("filepaths: {:?}", filepaths);
    if filepaths.is_empty() {
        warn!("No files found");
        return Ok(());
    }
    lint_filepaths(filepaths, args.fix).await;
    Ok(())
}
