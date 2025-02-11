use std::os::raw::{c_char, c_int};

use rusqlite::ffi::SQLITE_NOTICE;
use rusqlite::trace::log;
use rusqlite::{ffi, Connection, Result};

#[allow(unsafe_code)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn sqlite3_extension_init(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    Connection::extension_init2(db, pz_err_msg, p_api, extension_init)
}

fn extension_init(db: Connection) -> Result<bool> {
    utiles::sqlite_utiles::register_utiles_sqlite(&db)?;
    log(SQLITE_NOTICE, "Loaded sqlite-utiles extension");
    Ok(false)
}
