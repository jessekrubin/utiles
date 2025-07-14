use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    Debug,
    Default,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    strum_macros::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum MbtType {
    #[default]
    Flat,
    Hash,
    Norm,
    Tippecanoe,
    Planetiler,
    Unknown,
}

impl MbtType {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Flat => "flat",
            Self::Hash => "hash",
            Self::Norm => "norm",
            Self::Tippecanoe => "tippecanoe",
            Self::Planetiler => "planetiler",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for MbtType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s.to_ascii_lowercase().as_str() {
            "flat" => Self::Flat,
            "hash" | "flat-with-hash" => Self::Hash,
            "norm" | "normalized" => Self::Norm,
            "tippecanoe" => Self::Tippecanoe,
            "planetiler" => Self::Planetiler,
            _ => Self::Unknown,
        };
        Ok(t)
    }
}
