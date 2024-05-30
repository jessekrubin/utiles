use rusqlite::{Connection, Result as RusqliteResult};

use crate::sqlite::{
    analyze, is_empty_db, pragma_index_list, pragma_table_list, vacuum, vacuum_into,
    PragmaIndexListRow, PragmaTableListRow,
};

pub trait Sqlike3 {
    fn conn(&self) -> &Connection;

    fn is_empty_db(&self) -> RusqliteResult<bool> {
        is_empty_db(self.conn())
    }

    fn vacuum(&self) -> RusqliteResult<usize> {
        vacuum(self.conn())
    }

    fn vacuum_into(&self, dst: String) -> RusqliteResult<usize> {
        vacuum_into(self.conn(), dst)
    }

    fn analyze(&self) -> RusqliteResult<usize> {
        analyze(self.conn())
    }

    fn pragma_index_list(
        &self,
        table: &str,
    ) -> RusqliteResult<Vec<PragmaIndexListRow>> {
        pragma_index_list(self.conn(), table)
    }
    fn pragma_table_list(&self) -> RusqliteResult<Vec<PragmaTableListRow>> {
        pragma_table_list(self.conn())
    }
}
