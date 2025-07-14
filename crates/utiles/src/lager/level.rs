use crate::UtilesError;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LagerLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Default for LagerLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for LagerLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

impl FromStr for LagerLevel {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => {
                let e = format!(
                    "invalid lager level '{s}' (expected one of: trace, debug, info, warn, error)"
                );
                Err(UtilesError::AdHoc(e))
            }
        }
    }
}
