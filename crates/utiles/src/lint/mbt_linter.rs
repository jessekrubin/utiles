use std::path::{Path, PathBuf};

use tracing::warn;

use crate::errors::UtilesResult;
use crate::lint::MbtLint;
use crate::mbt::mbtiles::{
    has_unique_index_on_metadata, metadata_table_name_is_primary_key,
};
use crate::mbt::metadata2duplicates;
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::sqlite::AsyncSqliteConn;
use crate::UtilesError;

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
    async fn open_mbtiles(&self) -> UtilesResult<MbtilesClientAsync> {
        let pth = self.path.to_str().map_or_else(
            || Err(UtilesError::PathConversionError("path".to_string())),
            Ok,
        )?;
        let mbtiles = MbtilesClientAsync::open_readonly(pth).await?;
        Ok(mbtiles)
    }

    pub async fn check_magic_number(
        mbt: &MbtilesClientAsync,
    ) -> UtilesResult<Option<MbtLint>> {
        let magic_number_res = mbt.magic_number().await;
        match magic_number_res {
            Ok(magic_number) => {
                if magic_number == 0x4d50_4258 {
                    Ok(None)
                } else if magic_number == 0 {
                    Ok(Some(MbtLint::MissingMagicNumber))
                } else {
                    Ok(Some(MbtLint::UnknownMagicNumber(magic_number)))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn check_encoding(
        mbt: &MbtilesClientAsync,
    ) -> UtilesResult<Vec<MbtLint>> {
        let encoding = mbt.pragma_encoding().await?;
        if encoding.to_lowercase() != "utf-8" {
            return Ok(vec![MbtLint::EncodingNotUtf8(encoding)]);
        }
        Ok(vec![])
    }

    pub async fn check_metadata_rows(
        mbt: &MbtilesClientAsync,
    ) -> UtilesResult<Vec<MbtLint>> {
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
            .map(|k| MbtLint::MissingMetadataKv(k.clone()))
            .collect::<Vec<MbtLint>>();
        Ok(errs)
    }

    pub async fn check_metadata(
        mbt: &MbtilesClientAsync,
    ) -> UtilesResult<Vec<MbtLint>> {
        // that metadata table exists
        let has_unique_index_on_metadata_name = mbt
            .conn(has_unique_index_on_metadata)
            .await
            .map_err(UtilesError::SqliteError)?;

        let mut errs = vec![];
        let name_is_pk = mbt
            .conn(metadata_table_name_is_primary_key)
            .await
            .map_err(UtilesError::SqliteError)?;
        if has_unique_index_on_metadata_name || name_is_pk {
            let rows = mbt.metadata_rows().await?;
            let duplicate_rows = metadata2duplicates(rows.clone());
            if !duplicate_rows.is_empty() {
                errs.extend(
                    duplicate_rows
                        .keys()
                        .map(|k| MbtLint::DuplicateMetadataKey(k.clone()))
                        .collect::<Vec<MbtLint>>(),
                );
            }
        } else {
            errs.push(MbtLint::MissingUniqueIndex("metadata.name".to_string()));
        }

        let rows_errors = MbtilesLinter::check_metadata_rows(mbt).await?;
        errs.extend(rows_errors);
        Ok(errs)
    }

    pub async fn lint(&self) -> UtilesResult<Vec<MbtLint>> {
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

        let encoding_res = MbtilesLinter::check_encoding(&mbt).await?;
        lint_results.extend(encoding_res);

        let metadata_res = MbtilesLinter::check_metadata(&mbt).await?;
        lint_results.extend(metadata_res);
        let lint_errors = lint_results.into_iter().collect::<Vec<MbtLint>>();
        Ok(lint_errors)
    }
}
