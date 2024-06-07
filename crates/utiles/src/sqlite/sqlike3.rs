use crate::sqlite::errors::SqliteResult;
use crate::sqlite::page_size::pragma_page_size_get;
use crate::sqlite::{
    analyze, is_empty_db, pragma_freelist_count, pragma_index_list, pragma_page_count,
    pragma_page_size_set, pragma_table_list, vacuum, vacuum_into, PragmaIndexListRow,
    PragmaTableListRow,
};

macro_rules! sqlike3_methods {
    (
        trait $trait_name:ident {
            $($fn_name:ident($($arg_name:ident: $arg_type:ty),*) -> $ret_type:ty => $fn_impl:path;)*
        }
    ) => {
        pub trait $trait_name {
            fn conn(&self) -> &rusqlite::Connection;

            $(
                fn $fn_name(&self, $($arg_name: $arg_type),*) -> $ret_type {
                    $fn_impl(self.conn(), $($arg_name),*).map_err(Into::into)
                }
            )*
        }
    };
}
sqlike3_methods! {
    trait Sqlike3 {
        analyze() -> SqliteResult<usize> => analyze;
        is_empty_db() -> SqliteResult<bool> => is_empty_db;
        pragma_index_list(table: &str) -> SqliteResult<Vec<PragmaIndexListRow>> => pragma_index_list;
        pragma_page_count() -> SqliteResult<i64> => pragma_page_count;
        pragma_freelist_count() -> SqliteResult<i64> => pragma_freelist_count;
        pragma_page_size() -> SqliteResult<i64> => pragma_page_size_get;
        pragma_page_size_set(page_size: i64) -> SqliteResult<i64> => pragma_page_size_set;
        pragma_table_list() -> SqliteResult<Vec<PragmaTableListRow>> => pragma_table_list;
        vacuum() -> SqliteResult<usize> => vacuum;
        vacuum_into(dst: String) -> SqliteResult<usize> => vacuum_into;
    }
}
