use serde::Serialize;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, strum_macros::Display,
)]
pub enum InsertStrategy {
    #[default]
    None,
    Replace,
    Ignore,
    Rollback,
    Abort,
    Fail,
}

impl InsertStrategy {
    #[must_use]
    pub fn sql_prefix(&self) -> &str {
        match self {
            Self::None => "INSERT",
            Self::Replace => "INSERT OR REPLACE",
            Self::Ignore => "INSERT OR IGNORE",
            Self::Rollback => "INSERT OR ROLLBACK",
            Self::Abort => "INSERT OR ABORT",
            Self::Fail => "INSERT OR FAIL",
        }
    }

    #[must_use]
    pub fn requires_check(&self) -> bool {
        !matches!(self, Self::Replace | Self::Ignore)
    }
}
