use crate::UtilesError;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LagerFormat {
    Full = 0,
    Json = 1,
}

impl Default for LagerFormat {
    fn default() -> Self {
        Self::Full
    }
}

impl std::fmt::Display for LagerFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            LagerFormat::Full => write!(f, "full"),
            LagerFormat::Json => write!(f, "json"),
        }
    }
}

impl FromStr for LagerFormat {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(LagerFormat::Json),
            "full" => Ok(LagerFormat::Full),
            _ => Err(UtilesError::Str("invalid lager level".to_string())),
        }
    }
}
