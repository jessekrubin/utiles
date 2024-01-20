pub enum InsertStrategy {
    None,
    Replace,
    Ignore,
    Rollback,
    Abort,
    Fail,
}

impl InsertStrategy {
    pub fn to_sql_prefix(&self) -> &str {
        match self {
            InsertStrategy::None => "INSERT",
            InsertStrategy::Replace => "INSERT OR REPLACE",
            InsertStrategy::Ignore => "INSERT OR IGNORE",
            InsertStrategy::Rollback => "INSERT OR ROLLBACK",
            InsertStrategy::Abort => "INSERT OR ABORT",
            InsertStrategy::Fail => "INSERT OR FAIL",
        }
    }
}
