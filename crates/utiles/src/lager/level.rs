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
            LagerLevel::Error => write!(f, "error"),
            LagerLevel::Warn => write!(f, "warn"),
            LagerLevel::Info => write!(f, "info"),
            LagerLevel::Debug => write!(f, "debug"),
            LagerLevel::Trace => write!(f, "trace"),
        }
    }
}

impl FromStr for LagerLevel {
    type Err = UtilesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LagerLevel::Trace),
            "debug" => Ok(LagerLevel::Debug),
            "info" => Ok(LagerLevel::Info),
            "warn" => Ok(LagerLevel::Warn),
            "error" => Ok(LagerLevel::Error),
            _ => {
                let e = format!(
                    "invalid lager level '{s}' (expected one of: trace, debug, info, warn, error)"
                );
                Err(UtilesError::AdHoc(e))
            }
        }
    }
}
