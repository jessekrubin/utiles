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
            InsertStrategy::None => "INSERT",
            InsertStrategy::Replace => "INSERT OR REPLACE",
            InsertStrategy::Ignore => "INSERT OR IGNORE",
            InsertStrategy::Rollback => "INSERT OR ROLLBACK",
            InsertStrategy::Abort => "INSERT OR ABORT",
            InsertStrategy::Fail => "INSERT OR FAIL",
        }
    }

    #[must_use]
    pub fn requires_check(&self) -> bool {
        !matches!(self, InsertStrategy::Replace | InsertStrategy::Ignore)
    }
}
