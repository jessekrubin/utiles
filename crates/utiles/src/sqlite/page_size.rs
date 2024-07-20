//! Page size tools for sqlite

/// Return true if `page_size` is valid; power of 2 between 512 and 65536.
///
/// Ref: [SQLite Page Size](https://www.sqlite.org/pragma.html#pragma_page_size)
#[must_use]
#[inline]
pub fn is_valid_page_size(page_size: i64) -> bool {
    page_size == 512
        || page_size == 1024
        || page_size == 2048
        || page_size == 4096
        || page_size == 8192
        || page_size == 16384
        || page_size == 32768
        || page_size == 65536
}
