use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Projection {
    Geographic,
    Mercator,
}

impl Projection {
    pub fn to_string(&self) -> String {
        match self {
            Projection::Geographic => "geographic".to_string(),
            Projection::Mercator => "mercator".to_string(),
        }
    }
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
