use std::path::{Path, PathBuf};

use tracing::debug;

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::lint::MbtLint;
use crate::mbt::mbtiles::{
    delete_metadata_duplicate_key_values, has_unique_index_on_metadata,
    metadata_table_name_is_primary_key,
};
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::mbt::{metadata2duplicate_keys, metadata2duplicates};
use crate::sqlite::AsyncSqliteConn;

/// A trait for all lint rules.
/// - `check` detects problems and returns zero or more `MbtLint` results.
/// - `fix` tries to fix them (if `fix` logic is supported for that rule).
#[async_trait::async_trait]
pub(super) trait MbtLintRule {
    /// Returns the name or short description of this rule.
    fn name(&self) -> &'static str;

    /// Check the `MBTiles`, returning any lint findings.
    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>>;

    /// Optionally fix the `MBTiles`. If a rule cannot fix, this can be a no-op or an `Err`.
    async fn fix(&self, _mbt: &MbtilesClientAsync) -> UtilesResult<()> {
        // default: do nothing
        Ok(())
    }
}

/// A rule that checks the `MBTiles` magic number.
pub(super) struct MagicNumberRule;

#[async_trait::async_trait]
impl MbtLintRule for MagicNumberRule {
    fn name(&self) -> &'static str {
        "MagicNumberRule"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        let mut results = vec![];
        match mbt.magic_number().await {
            Ok(magic_number) => {
                if magic_number == 0x4d50_4258 {
                    // No problem
                } else if magic_number == 0 {
                    results.push(MbtLint::MissingMagicNumber);
                } else {
                    results.push(MbtLint::UnknownMagicNumber(magic_number));
                }
            }
            Err(e) => return Err(e),
        }
        Ok(results)
    }

    async fn fix(&self, mbt: &MbtilesClientAsync) -> UtilesResult<()> {
        mbt.conn(|conn| {
            conn.execute_batch("PRAGMA application_id = 0x4d504258;")?;
            Ok(())
        })
        .await?;

        Ok(())
    }
}

/// A rule that checks whether the encoding is UTF-8.
pub(super) struct EncodingRule;

#[async_trait::async_trait]
impl MbtLintRule for EncodingRule {
    fn name(&self) -> &'static str {
        "EncodingRule"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        let encoding = mbt.pragma_encoding().await?;
        if encoding.to_lowercase() != "utf-8" {
            return Ok(vec![MbtLint::EncodingNotUtf8(encoding)]);
        }
        Ok(vec![])
    }

    async fn fix(&self, mbt: &MbtilesClientAsync) -> UtilesResult<()> {
        mbt.conn(|conn| {
            conn.execute_batch("PRAGMA encoding = 'UTF-8';")?;
            Ok(())
        })
        .await?;

        Ok(())
    }
}

pub(super) struct MetadataDuplicateKeyValues;

#[async_trait::async_trait]
impl MbtLintRule for MetadataDuplicateKeyValues {
    fn name(&self) -> &'static str {
        "metadata-duplicate-key-values"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        let rows = mbt.metadata_duplicate_key_values().await?;
        let violations = rows
            .into_iter()
            .map(|(key, value, count)| {
                MbtLint::DuplicateMetadataKeyValue(key, value, count)
            })
            .collect();
        Ok(violations)
    }

    async fn fix(&self, mbt: &MbtilesClientAsync) -> UtilesResult<()> {
        let res = mbt
            .conn(|conn| {
                let r = delete_metadata_duplicate_key_values(conn)?;
                Ok(r)
            })
            .await?;
        debug!("Deleted {} duplicate metadata key-values", res);
        Ok(())
    }
}

pub(super) struct MetadataDuplicateKeys;

#[async_trait::async_trait]
impl MbtLintRule for MetadataDuplicateKeys {
    fn name(&self) -> &'static str {
        "metadata-duplicate-keys"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        // only get the keys and values that have matching keys but different values
        let rows = mbt.metadata_rows().await?;

        let duplicate_rows = metadata2duplicates(rows.clone());
        if !duplicate_rows.is_empty() {
            // convert to map of key -> list[values]
            let map = metadata2duplicate_keys(rows);
            let violations = map
                .into_iter()
                .map(|(key, values)| {
                    let values_str = values
                        .into_iter()
                        .map(|v| v.value)
                        .collect::<Vec<String>>()
                        .join(", ");

                    MbtLint::DuplicateMetadataKey(key, values_str)
                })
                .collect();
            return Ok(violations);
        }
        Ok(vec![])
    }
}

pub(super) struct MetadataUniqueIndex;

#[async_trait::async_trait]
impl MbtLintRule for MetadataUniqueIndex {
    fn name(&self) -> &'static str {
        "metadata-unique-index"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        let has_unique_index_on_metadata_name = mbt
            .conn(has_unique_index_on_metadata)
            .await
            .map_err(UtilesError::SqliteError)?;
        let name_is_pk = mbt
            .conn(metadata_table_name_is_primary_key)
            .await
            .map_err(UtilesError::SqliteError)?;
        if !has_unique_index_on_metadata_name && !name_is_pk {
            let lint_violations =
                vec![MbtLint::MissingUniqueIndex("metadata.name".to_string())];
            return Ok(lint_violations);
        }
        Ok(vec![])
    }

    async fn fix(&self, mbt: &MbtilesClientAsync) -> UtilesResult<()> {
        // if all metadata keys are unique, we can add a unique index
        let rows = mbt.metadata_rows().await?;
        let duplicate_rows = metadata2duplicates(rows.clone());
        if duplicate_rows.is_empty() {
            let res = mbt
                .conn(|conn| {
                    conn.execute_batch(
                        "CREATE UNIQUE INDEX metadata_index ON metadata (name);",
                    )?;
                    Ok(())
                })
                .await?;
            debug!("Added unique index on metadata.name: {:?}", res);
        }
        Ok(())
    }
}

/// A rule that checks for required metadata fields & duplicates.
pub(super) struct MetadataRequiredKeysRule;

#[async_trait::async_trait]
impl MbtLintRule for MetadataRequiredKeysRule {
    fn name(&self) -> &'static str {
        "MetadataRule"
    }

    async fn check(&self, mbt: &MbtilesClientAsync) -> UtilesResult<Vec<MbtLint>> {
        let mut errs = vec![];

        let metadata_rows = mbt.metadata_rows().await?;
        let metadata_keys = metadata_rows
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        let missing_metadata_keys = crate::lint::REQUIRED_METADATA_FIELDS
            .iter()
            .filter(|k| !metadata_keys.contains(&(*(*k)).to_string()))
            .map(|k| (*k).to_string())
            .collect::<Vec<String>>();

        for k in missing_metadata_keys {
            errs.push(MbtLint::MissingMetadataKv(k));
        }

        Ok(errs)
    }
}

#[derive(Debug)]
pub(super) struct MbtilesLinter {
    pub path: PathBuf,
    pub fix: bool,
}

impl MbtilesLinter {
    #[must_use]
    pub(super) fn new<T: AsRef<Path>>(path: T, fix: bool) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            fix,
        }
    }

    async fn open_mbtiles(&self) -> UtilesResult<MbtilesClientAsync> {
        let pth = self.path.to_str().map_or_else(
            || Err(UtilesError::PathConversionError("path".to_string())),
            Ok,
        )?;
        if self.fix {
            MbtilesClientAsync::open_existing(pth).await
        } else {
            MbtilesClientAsync::open_readonly(pth).await
        }
    }

    /// Gather all your rules in one place.
    /// - In the future, you can push new rules into this vector
    ///   more easily.
    fn lint_rules() -> Vec<Box<dyn MbtLintRule + Send + Sync>> {
        vec![
            Box::new(MagicNumberRule),
            Box::new(MetadataDuplicateKeyValues),
            Box::new(MetadataDuplicateKeys),
            Box::new(MetadataUniqueIndex),
            Box::new(EncodingRule),
            Box::new(MetadataRequiredKeysRule),
            // Additional rules go here...
        ]
    }

    /// Run all rules; collect their errors; optionally fix if `self.fix` is `true`.
    pub(super) async fn lint(&self) -> UtilesResult<Vec<MbtLint>> {
        let mbt = self.open_mbtiles().await?;

        // If not an MBTiles-like DB, exit early
        if !mbt.is_mbtiles_like().await? {
            let pth = self.path.to_string_lossy().into_owned();
            return Err(UtilesError::NonMbtilesSqliteDb(pth));
        }

        let rules = Self::lint_rules();
        let mut all_errors = vec![];

        if self.fix {
            for rule in &rules {
                debug!("Checking rule: {}", rule.name());
                // first we check!
                let errs = rule.check(&mbt).await?;
                if !errs.is_empty() {
                    // attempt the fix here...
                    rule.fix(&mbt).await?;
                }
                // post fix check
                let mut errs = rule.check(&mbt).await?;
                all_errors.append(&mut errs);
            }
        } else {
            for rule in &rules {
                // 1) Check
                let mut errs = rule.check(&mbt).await?;
                all_errors.append(&mut errs);
            }
        }

        Ok(all_errors)
    }
}
