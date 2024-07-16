use std::path::{Path, PathBuf};

use crate::mbt::metadata2duplicates;
use crate::sqlite::AsyncSqliteConn;
use crate::utilesqlite::mbtiles::{
    has_unique_index_on_metadata, metadata_table_name_is_primary_key,
};
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::{utilesqlite, UtilesError, UtilesResult};
use colored::Colorize;
use futures::{stream, StreamExt};
use thiserror::Error;
use tracing::{debug, warn};

pub const REQUIRED_METADATA_FIELDS: [&str; 5] =
    ["bounds", "format", "maxzoom", "minzoom", "name"];

#[derive(Error, Debug)]
pub enum UtilesLintError {
    #[error("unable to open: {0}")]
    UnableToOpen(String),

    #[error("not a sqlite database error: {0}")]
    NotASqliteDb(String),

    #[error("not a mbtiles database error: {0}")]
    NotAMbtilesDb(String),

    #[error("no tiles table/view")]
    MbtMissingTiles,

    #[error("no metadata table/view")]
    MbtMissingMetadata,

    #[error("missing mbtiles magic-number/application_id")]
    MbtMissingMagicNumber,

    #[error("Unrecognized mbtiles magic-number/application_id: {0} != 0x4d504258")]
    MbtUnknownMagicNumber(u32),

    #[error("missing index: {0}")]
    MissingIndex(String),

    #[error("missing unique index: {0}")]
    MissingUniqueIndex(String),

    #[error("duplicate metadata key: {0}")]
    DuplicateMetadataKey(String),

    #[error("metadata k/v missing: {0}")]
    MbtMissingMetadataKv(String),

    #[error("unknown error: {0}")]
    Unknown(String),

    #[error("lint errors {0:?}")]
    LintErrors(Vec<UtilesLintError>),

    #[error("utiles error: {0}")]
    UtilesError(#[from] UtilesError),
}

impl UtilesLintError {
    #[must_use]
    pub fn format_error(&self, filepath: &str) -> String {
        let errcode = "MBT".red();
        let errstr = format!("{errcode}: {self}");

        // let error_str = format!("STTUFF -- {}", self.to_string());
        let e_str = format!("{filepath}: {errstr}");
        e_str
        // match self {
        //     UtilesError::CoreError(e) => e.to_string(),
        //     UtilesError::Unimplemented(e) => e.to_string(),
        //     UtilesError::SqliteError(e) => e.to_string(),
        //     UtilesError::AsyncSqliteError(e) => e.to_string(),
        //     UtilesError::FileDoesNotExist(e) => e.to_string(),
        //     UtilesError::ParseIntError(e) => e.to_string(),
        //     UtilesError::Error(e) => e.to_string(),
        //     UtilesError::Unknown(e) => e.to_string(),
        // }
    }
}

// impl From<UtilesError> for UtilesLintError {
//     fn from(e: UtilesError) -> Self {
//         UtilesLintError::UtilesError(e)
//     }
// }

pub type UtilesLintResult<T> = Result<T, UtilesLintError>;

#[derive(Debug)]
pub struct MbtilesLinter {
    pub path: PathBuf,
    pub fix: bool,
}

impl MbtilesLinter {
    #[must_use]
    pub fn new<T: AsRef<Path>>(path: T, fix: bool) -> Self {
        MbtilesLinter {
            path: path.as_ref().to_path_buf(),
            fix,
        }
    }
    async fn open_mbtiles(
        &self,
    ) -> UtilesResult<utilesqlite::MbtilesAsyncSqliteClient> {
        let pth = self.path.to_str().map_or_else(
            || Err(UtilesError::PathConversionError("path".to_string())),
            Ok,
        )?;
        let mbtiles = utilesqlite::MbtilesAsyncSqliteClient::open_readonly(pth).await?;
        Ok(mbtiles)
    }

    pub async fn check_magic_number(
        mbt: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<Option<UtilesLintError>> {
        let magic_number_res = mbt.magic_number().await;
        match magic_number_res {
            Ok(magic_number) => {
                if magic_number == 0x4d50_4258 {
                    Ok(None)
                } else if magic_number == 0 {
                    Ok(Some(UtilesLintError::MbtMissingMagicNumber))
                } else {
                    Ok(Some(UtilesLintError::MbtUnknownMagicNumber(magic_number)))
                }
            }
            Err(e) => Err(e),
        }
    }
    pub async fn check_metadata_rows(
        mbt: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<Option<UtilesLintError>> {
        let metadata_rows = mbt.metadata_rows().await?;
        let metadata_keys = metadata_rows
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        let missing_metadata_keys = REQUIRED_METADATA_FIELDS
            .iter()
            .filter(|k| !metadata_keys.contains(&(**k).to_string()))
            .map(|k| (*k).to_string())
            .collect::<Vec<String>>();
        if missing_metadata_keys.is_empty() {
            Ok(Some(UtilesLintError::MbtMissingMetadataKv(
                "metadata".to_string(),
            )))
        } else {
            let errs = missing_metadata_keys
                .iter()
                .map(|k| UtilesLintError::MbtMissingMetadataKv(k.clone()))
                .collect::<Vec<UtilesLintError>>();
            Ok(Some(UtilesLintError::LintErrors(errs)))
        }
    }

    pub async fn check_metadata(
        mbt: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<Vec<UtilesLintError>> {
        // that metadata table exists
        let has_unique_index_on_metadata_name = mbt
            .conn(has_unique_index_on_metadata)
            .await
            .map_err(UtilesError::AsyncSqliteError)?;

        let mut errs = vec![];
        let name_is_pk = mbt
            .conn(metadata_table_name_is_primary_key)
            .await
            .map_err(UtilesError::AsyncSqliteError)?;
        if has_unique_index_on_metadata_name || name_is_pk {
            let rows = mbt.metadata_rows().await?;
            let duplicate_rows = metadata2duplicates(rows.clone());
            if !duplicate_rows.is_empty() {
                errs.extend(
                    duplicate_rows
                        .keys()
                        .map(|k| UtilesLintError::DuplicateMetadataKey(k.clone()))
                        .collect::<Vec<UtilesLintError>>(),
                );
            }
        } else {
            errs.push(UtilesLintError::MissingUniqueIndex(
                "metadata.name".to_string(),
            ));
        }

        // let rows_errs = MbtilesLinter::check_metadata_rows(mbt).await?;
        if let Some(e) = MbtilesLinter::check_metadata_rows(mbt).await? {
            match e {
                UtilesLintError::LintErrors(es) => {
                    errs.extend(es);
                }
                _ => {
                    errs.push(e);
                }
            }
        }
        Ok(errs)
    }

    pub async fn lint(&self) -> UtilesResult<Vec<UtilesLintError>> {
        let mbt = self.open_mbtiles().await?;
        if !mbt.is_mbtiles_like().await? {
            let pth = self.path.to_str().unwrap_or("unknown-path");
            return Err(UtilesError::NonMbtilesSqliteDb(pth.to_string()));
        }
        let mut lint_results = vec![];
        // lint_results.push(MbtilesLinter::check_magic_number(&mbt).await);
        let metadata_res = MbtilesLinter::check_metadata(&mbt).await?;
        lint_results.extend(metadata_res);
        let lint_errors = lint_results.into_iter().collect::<Vec<UtilesLintError>>();
        Ok(lint_errors)
    }
}
// impl MbtilesLinter {
//     #[must_use]
//     pub fn new<T: AsRef<Path>>(path: T, fix: bool) -> Self {
//         MbtilesLinter {
//             path: path.as_ref().to_path_buf(),
//             fix,
//         }
//     }
//     async fn open_mbtiles(
//         &self,
//     ) -> UtilesLintResult<MbtilesAsyncSqliteClient> {
//         let pth = self.path.to_str();
//         let mbtiles =
//             match MbtilesAsyncSqliteClient::open_existing(pth).await {
//                 Ok(m) => m,
//                 Err(e) => {
//                     return Err(UtilesLintError::UnableToOpen(e.to_string()));
//                 }
//             };
//         Ok(mbtiles)
//     }
//
//     pub async fn check_magic_number(
//         mbt: &MbtilesAsyncSqliteClient,
//     ) -> UtilesLintResult<()> {
//         let magic_number_res = mbt.magic_number().await;
//         match magic_number_res {
//             Ok(magic_number) => {
//                 if magic_number == 0x4d50_4258 {
//                     Ok(())
//                 } else if magic_number == 0 {
//                     Err(UtilesLintError::MbtMissingMagicNumber)
//                 } else {
//                     Err(UtilesLintError::MbtUnknownMagicNumber(magic_number))
//                 }
//             }
//             Err(e) => Err(e.into()),
//         }
//     }
//     pub async fn check_metadata_rows(
//         mbt: &MbtilesAsyncSqliteClient,
//     ) -> UtilesLintResult<()> {
//         let metadata_rows = mbt.metadata_rows().await?;
//         let metadata_keys = metadata_rows
//             .iter()
//             .map(|r| r.name.clone())
//             .collect::<Vec<String>>();
//         let missing_metadata_keys = REQUIRED_METADATA_FIELDS
//             .iter()
//             .filter(|k| !metadata_keys.contains(&(**k).to_string()))
//             .map(|k| (*k).to_string())
//             .collect::<Vec<String>>();
//         if missing_metadata_keys.is_empty() {
//             Ok(())
//         } else {
//             let errs = missing_metadata_keys
//                 .iter()
//                 .map(|k| UtilesLintError::MbtMissingMetadataKv(k.clone()))
//                 .collect::<Vec<UtilesLintError>>();
//             Err(UtilesLintError::LintErrors(errs))
//         }
//     }
//
//     pub async fn check_metadata(
//         mbt: &MbtilesAsyncSqliteClient,
//     ) -> UtilesLintResult<()> {
//         // that metadata table exists
//         let has_unique_index_on_metadata_name = mbt
//             .conn(has_unique_index_on_metadata)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)?;
//
//         let mut errs = vec![];
//         let name_is_pk = mbt
//             .conn(metadata_table_name_is_primary_key)
//             .await
//             .map_err(UtilesError::AsyncSqliteError)?;
//         if has_unique_index_on_metadata_name || name_is_pk {
//             let rows = mbt.metadata_rows().await?;
//             let duplicate_rows = metadata2duplicates(rows.clone());
//             if !duplicate_rows.is_empty() {
//                 errs.extend(
//                     duplicate_rows
//                         .keys()
//                         .map(|k| UtilesLintError::DuplicateMetadataKey(k.clone()))
//                         .collect::<Vec<UtilesLintError>>(),
//                 );
//             }
//         } else {
//             errs.push(UtilesLintError::MissingUniqueIndex(
//                 "metadata.name".to_string(),
//             ));
//         }
//
//         let rows_errs = MbtilesLinter::check_metadata_rows(mbt).await;
//         if let Err(e) = rows_errs {
//             match e {
//                 UtilesLintError::LintErrors(es) => {
//                     errs.extend(es);
//                 }
//                 _ => {
//                     errs.push(e);
//                 }
//             }
//         }
//         if errs.is_empty() {
//             Ok(())
//         } else {
//             Err(UtilesLintError::LintErrors(errs))
//         }
//     }
//
//     pub async fn lint(&self) -> UtilesLintResult<Vec<UtilesLintError>> {
//         let mbt = self.open_mbtiles().await?;
//
//         // let r = mbt
//         //     .conn_mut(|c| {
//         //         c.trace(Some(|s| {
//         //             warn!("stmt: {}", s);
//         //         }));
//         //         Ok(())
//         //     })
//         //     .await;
//         // info!("trace: {:?}", r);
//
//         // let prof = mbt
//         //     .conn_mut(|c| {
//         //         c.profile(Some(|s, durr| {
//         //             info!("SQL (dt: {:?}): {}", durr, s);
//         //         }));
//         //         Ok(())
//         //     })
//         //     .await;
//
//         // let query_res = mbt.conn(metadata_duplicates_json_query).await;
//         // info!("query_res: {:?}", query_res);
//         if !mbt.is_mbtiles_like().await? {
//             let pth = self.path.to_str().unwrap_or("unknown-path");
//             return Err(UtilesLintError::NotAMbtilesDb(pth.to_string()));
//         }
//         let mut lint_results = vec![];
//         // lint_results.push(MbtilesLinter::check_magic_number(&mbt).await);
//
//         match MbtilesLinter::check_metadata(&mbt).await {
//             Ok(()) => {}
//             Err(e) => match e {
//                 UtilesLintError::LintErrors(errs) => {
//                     lint_results.push(Ok(()));
//                     lint_results.extend(errs.into_iter().map(Err));
//                 }
//                 _ => {
//                     lint_results.push(Err(e));
//                 }
//             },
//         }
//
//         Ok(lint_results
//             .into_iter()
//             .filter_map(std::result::Result::err)
//             .collect())
//     }
// }

pub async fn lint_filepaths(fspaths: Vec<PathBuf>, fix: bool) {
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
                        debug!("{} - {} errors found", path.display(), r.len());
                        // let agg_err = UtilesLintError::LintErrors(r);
                        let strings = r
                            .iter()
                            .map(|e| e.format_error(&path.display().to_string()))
                            .collect::<Vec<String>>();
                        let joined = strings.join("\n");
                        println!("{joined}");
                        for err in r {
                            debug!("{}", err.to_string());
                        }
                    }
                }
                Err(e) => {
                    let e_msg = format!("{}: {}", path.display(), e);
                    warn!("{}", e_msg);
                }
            }
        })
        .await;
}
