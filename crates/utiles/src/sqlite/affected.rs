use serde::{Deserialize, Serialize};

#[derive(Debug, strum::Display, Serialize, Deserialize, strum::EnumString)]
// #[serde(rename_all = "lowercase")]
pub enum AffectedType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RowsAffected {
    #[serde(rename = "type")]
    pub type_: AffectedType,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub table: Option<String>,
    pub count: usize,
}
