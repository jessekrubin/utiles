//! Utiles configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LintConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,

    pub rules: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilesConfig {
    pub lint: LintConfig,
    // TODO: server/log config
    // pub log: LagerConfig,
    // pub serve : ServeConfig,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ServeConfig {
//     pub host: String,
//     pub port: u16,
// }
//
// #[derive(Debug, Deserialize, Serialize)]
// pub struct LagerConfig {
//     pub level: String,
//     pub json: bool,
// }

// impl Default for LagerConfig {
//     fn default() -> Self {
//         Self {
//             level: "info".to_string(),
//             json: false,
//         }
//     }
// }
