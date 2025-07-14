use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::UtilesError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum HashType {
    Md5,
    Fnv1a,
    Xxh32,
    #[default]
    Xxh64,
    Xxh3_64,
    Xxh3_128,
}

impl std::fmt::Display for HashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Md5 => write!(f, "md5"),
            Self::Fnv1a => write!(f, "fnv1a"),
            Self::Xxh32 => write!(f, "xxh32"),
            Self::Xxh64 => write!(f, "xxh64"),
            Self::Xxh3_64 => write!(f, "xxh3_64"),
            Self::Xxh3_128 => write!(f, "xxh3_128"),
        }
    }
}

impl FromStr for HashType {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "md5" => Ok(Self::Md5),
            "fnv" | "fnv1a" | "fnv1a64" => Ok(Self::Fnv1a),
            "xxh32" => Ok(Self::Xxh32),
            "xxh64" => Ok(Self::Xxh64),
            "xxh3" | "xxh3_64" | "xxh3-64" => Ok(Self::Xxh3_64),
            "xxh3_128" | "xxh3-128" => Ok(Self::Xxh3_128),
            _ => Err(UtilesError::Error(format!(
                "HashType::from_str: unknown hash type: {s}"
            ))),
        }
    }
}

impl HashType {
    #[must_use]
    pub fn sqlite_fn_name(&self) -> &'static str {
        match self {
            Self::Md5 => "md5",
            Self::Fnv1a => "fnv1a",
            Self::Xxh32 => "xxh32",
            Self::Xxh64 => "xxh64",
            Self::Xxh3_64 => "xxh3_64",
            Self::Xxh3_128 => "xxh3_128",
        }
    }

    #[must_use]
    pub fn sqlite_hex_fn_name(&self) -> &'static str {
        match self {
            Self::Md5 => "md5_hex",
            Self::Fnv1a => "fnv1a_hex",
            Self::Xxh32 => "xxh32_hex",
            Self::Xxh64 => "xxh64_hex",
            Self::Xxh3_64 => "xxh3_64_hex",
            Self::Xxh3_128 => "xxh3_128_hex",
        }
    }

    #[must_use]
    pub fn digest_size(&self) -> usize {
        match self {
            Self::Xxh32 => 4,
            Self::Md5 | Self::Xxh3_128 => 16,
            Self::Fnv1a | Self::Xxh64 | Self::Xxh3_64 => 8,
        }
    }
}
