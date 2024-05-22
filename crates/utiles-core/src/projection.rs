use crate::UtilesCoreError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Projection enum
#[derive(Debug, Serialize, Deserialize)]
pub enum Projection {
    /// Geographic projection (lat/lng coordinates)
    Geographic,

    /// Mercator projection (x/y coordinates)
    Mercator,
}

impl TryFrom<String> for Projection {
    type Error = UtilesCoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "geographic" => Ok(Projection::Geographic),
            "mercator" => Ok(Projection::Mercator),
            _ => Err(UtilesCoreError::Str(value)),
        }
    }
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Projection::Geographic => write!(f, "geographic"),
            Projection::Mercator => write!(f, "mercator"),
        }
    }
}
