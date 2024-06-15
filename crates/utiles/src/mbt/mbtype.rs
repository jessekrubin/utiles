use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[derive(clap::ValueEnum)]
pub enum MbtType {
    #[default]
    Flat,
    Hash,
    Norm,
    Tippecanoe,
    Unknown,
}

impl MbtType {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            MbtType::Flat => "flat",
            MbtType::Hash => "hash",
            MbtType::Norm => "norm",
            MbtType::Tippecanoe => "tippecanoe",
            MbtType::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for MbtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MbtType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s.to_ascii_lowercase().as_str() {
            "flat" => MbtType::Flat,
            "hash" | "flat-with-hash" => MbtType::Hash,
            "norm" | "normalized" => MbtType::Norm,
            "tippecanoe" => MbtType::Tippecanoe,
            _ => MbtType::Unknown,
        };
        Ok(t)
    }
}
