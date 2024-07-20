use crate::errors::UtilesResult;
use crate::lint::UtilesLintError;
use crate::mbt::metadata2duplicates;
use crate::sqlite::AsyncSqliteConn;
use crate::utilesqlite::mbtiles::{
    has_unique_index_on_metadata, metadata_table_name_is_primary_key,
};
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::{utilesqlite, UtilesError};
use std::path::{Path, PathBuf};
use tracing::warn;

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
    ) -> UtilesResult<Option<crate::lint::UtilesLintError>> {
        let magic_number_res = mbt.magic_number().await;
        match magic_number_res {
            Ok(magic_number) => {
                if magic_number == 0x4d50_4258 {
                    Ok(None)
                } else if magic_number == 0 {
                    Ok(Some(crate::lint::UtilesLintError::MbtMissingMagicNumber))
                } else {
                    Ok(Some(crate::lint::UtilesLintError::MbtUnknownMagicNumber(
                        magic_number,
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }
    pub async fn check_metadata_rows(
        mbt: &MbtilesAsyncSqliteClient,
    ) -> UtilesResult<Vec<UtilesLintError>> {
        let metadata_rows = mbt.metadata_rows().await?;
        let metadata_keys = metadata_rows
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        let missing_metadata_keys = crate::lint::REQUIRED_METADATA_FIELDS
            .iter()
            .filter(|k| !metadata_keys.contains(&(**k).to_string()))
            .map(|k| (*k).to_string())
            .collect::<Vec<String>>();
        let errs = missing_metadata_keys
            .iter()
            .map(|k| crate::lint::UtilesLintError::MbtMissingMetadataKv(k.clone()))
            .collect::<Vec<crate::lint::UtilesLintError>>();
        Ok(errs)
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

        let rows_errors = MbtilesLinter::check_metadata_rows(mbt).await?;
        errs.extend(rows_errors);
        Ok(errs)
    }

    pub async fn lint(&self) -> UtilesResult<Vec<crate::lint::UtilesLintError>> {
        if self.fix {
            warn!("Fix not implemented (yet)");
        }
        let mbt = self.open_mbtiles().await?;
        if !mbt.is_mbtiles_like().await? {
            let pth = self.path.to_str().unwrap_or("unknown-path");
            return Err(UtilesError::NonMbtilesSqliteDb(pth.to_string()));
        }
        let mut lint_results = vec![];
        let magic_res = MbtilesLinter::check_magic_number(&mbt).await?;
        if let Some(e) = magic_res {
            lint_results.push(e);
        }
        let metadata_res = MbtilesLinter::check_metadata(&mbt).await?;
        lint_results.extend(metadata_res);
        let lint_errors = lint_results
            .into_iter()
            .collect::<Vec<crate::lint::UtilesLintError>>();
        Ok(lint_errors)
    }
}
