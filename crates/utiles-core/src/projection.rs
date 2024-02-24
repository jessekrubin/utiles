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

impl From<String> for Projection {
    fn from(s: String) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "mercator" => Projection::Mercator,
            "geographic" => Projection::Geographic,
            _ => {
                panic!("Invalid projection: {s}");
            }
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
