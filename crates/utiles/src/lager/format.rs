use crate::UtilesError;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum LagerFormat {
    #[default]
    Full = 0,
    Json = 1,
}

impl std::fmt::Display for LagerFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Full => write!(f, "full"),
            Self::Json => write!(f, "json"),
        }
    }
}

impl FromStr for LagerFormat {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "full" => Ok(Self::Full),
            _ => Err(UtilesError::AdHoc("invalid lager level".to_string())),
        }
    }
}
