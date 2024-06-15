use crate::UtilesError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Md5,
    Fnv1a,
    Xxh32,
    Xxh64,
    Xxh3_64,
    Xxh3_128,
}

// display for HashType
impl std::fmt::Display for HashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashType::Md5 => write!(f, "md5"),
            HashType::Fnv1a => write!(f, "fnv1a"),
            HashType::Xxh32 => write!(f, "xxh32"),
            HashType::Xxh64 => write!(f, "xxh64"),
            HashType::Xxh3_64 => write!(f, "xxh3_64"),
            HashType::Xxh3_128 => write!(f, "xxh3_128"),
        }
    }
}

impl FromStr for HashType {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "md5" => Ok(HashType::Md5),
            "fnv1a" | "fnv1a64" => Ok(HashType::Fnv1a),
            "xxh32" => Ok(HashType::Xxh32),
            "xxh64" => Ok(HashType::Xxh64),
            "xxh3_64" => Ok(HashType::Xxh3_64),
            "xxh3_128" => Ok(HashType::Xxh3_128),
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
            HashType::Md5 => "md5",
            HashType::Fnv1a => "fnv1a",
            HashType::Xxh32 => "xxh32",
            HashType::Xxh64 => "xxh64",
            HashType::Xxh3_64 => "xxh3_64",
            HashType::Xxh3_128 => "xxh3_128",
        }
    }

    #[must_use]
    pub fn sqlite_hex_fn_name(&self) -> &'static str {
        match self {
            HashType::Md5 => "md5_hex",
            HashType::Fnv1a => "fnv1a_hex",
            HashType::Xxh32 => "xxh32_hex",
            HashType::Xxh64 => "xxh64_hex",
            HashType::Xxh3_64 => "xxh3_64_hex",
            HashType::Xxh3_128 => "xxh3_128_hex",
        }
    }

    #[must_use]
    pub fn digest_size(&self) -> usize {
        match self {
            HashType::Xxh32 => 4,
            HashType::Md5 | HashType::Xxh3_128 => 16,
            HashType::Fnv1a | HashType::Xxh64 | HashType::Xxh3_64 => 8,
        }
    }
}
