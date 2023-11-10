use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum Projection {
    Geographic,
    Mercator,
}

impl From<String> for Projection {
    fn from(s: String) -> Self {
        match s.as_str() {
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
