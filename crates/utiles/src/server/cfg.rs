//! Utiles server configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilesServerConfig {
    pub host: String,
    pub port: u16,
    pub fspaths: Vec<String>,
}

impl UtilesServerConfig {
    #[must_use]
    pub fn new(host: String, port: u16, fspaths: Vec<String>) -> Self {
        Self {
            host,
            port,
            fspaths,
        }
    }

    #[must_use]
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
