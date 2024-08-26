use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Health {
    status: String,
    uptime: u64,
}

impl Health {
    pub(crate) fn new(status: String, uptime: u64) -> Self {
        Self { status, uptime }
    }
}
