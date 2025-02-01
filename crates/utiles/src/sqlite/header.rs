//! Sqlite header structs/parsing
//!

use std::convert::TryInto;
use std::io::{self};

use serde::{Deserialize, Serialize};

use crate::sqlite::{SqliteError, SqliteResult};

/// Sqlite header struct:
///
/// - The header of a sqlite3 database file is 100 bytes long (BOOM).
/// - Has a bunch of field that are listed below!
/// - IS BEING PARSED HERE BECAUSE AN OLD CODE I KNOW TO FIDDLE WITH SQLITE HEADERS
///
/// Sqlite header fields (from [sqlite.org](https://www.sqlite.org/fileformat2.html#the_database_header)):
///
/// | Offset | Size | Description                                               |
/// |--------|------|-----------------------------------------------------------|
/// | 0      | 16   | The header string: "SQLite format 3\000"                  |
/// | 16     | 2    | The database page size in bytes. Must be a power of two between 512 and 32768 inclusive, or the value 1 representing a page size of 65536. |
/// | 18     | 1    | File format write version. 1 for legacy; 2 for WAL.       |
/// | 19     | 1    | File format read version. 1 for legacy; 2 for WAL.        |
/// | 20     | 1    | Bytes of unused "reserved" space at the end of each page. Usually 0. |
/// | 21     | 1    | Maximum embedded payload fraction. Must be 64.            |
/// | 22     | 1    | Minimum embedded payload fraction. Must be 32.            |
/// | 23     | 1    | Leaf payload fraction. Must be 32.                        |
/// | 24     | 4    | File change counter.                                      |
/// | 28     | 4    | Size of the database file in pages. The "in-header database size". |
/// | 32     | 4    | Page number of the first freelist trunk page.             |
/// | 36     | 4    | Total number of freelist pages.                           |
/// | 40     | 4    | The schema cookie.                                        |
/// | 44     | 4    | The schema format number. Supported schema formats are 1, 2, 3, and 4. |
/// | 48     | 4    | Default page cache size.                                  |
/// | 52     | 4    | The page number of the largest root b-tree page when in auto-vacuum or incremental-vacuum modes, or zero otherwise. |
/// | 56     | 4    | The database text encoding. A value of 1 means UTF-8. A value of 2 means UTF-16le. A value of 3 means UTF-16be. |
/// | 60     | 4    | The "user version" as read and set by the user_version pragma. |
/// | 64     | 4    | True (non-zero) for incremental-vacuum mode. False (zero) otherwise. |
/// | 68     | 4    | The "Application ID" set by PRAGMA application_id.        |
/// | 72     | 20   | Reserved for expansion. Must be zero.                     |
/// | 92     | 4    | The version-valid-for number.                             |
/// | 96     | 4    | SQLITE_VERSION_NUMBER                                     |
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SqliteHeader {
    #[serde(serialize_with = "serialize_magic_string")]
    magic_string: [u8; 16],
    page_size: u16,
    write_version: u8,
    read_version: u8,
    reserved_space: u8,
    max_payload_fraction: u8,
    min_payload_fraction: u8,
    leaf_payload_fraction: u8,
    file_change_counter: u32,
    database_size: u32,
    first_freelist_trunk_page: u32,
    total_freelist_pages: u32,
    schema_cookie: u32,
    schema_format_number: u32,
    default_page_cache_size: u32,
    largest_root_btree_page: u32,
    text_encoding: u32,
    user_version: u32,
    incremental_vacuum_mode: u32,
    application_id: u32,
    #[serde(skip)]
    reserved: [u8; 20],
    version_valid_for: u32,
    sqlite_version_number: u32,
}

pub fn serialize_magic_string<S>(
    magic_string: &[u8; 16],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&String::from_utf8_lossy(magic_string))
}

impl SqliteHeader {
    fn magic_ok(b: &[u8]) -> SqliteResult<()> {
        if b == b"SQLite format 3\0" {
            Ok(())
        } else {
            Err(SqliteError::InvalidSqliteMagic(format!("{b:?}")))
        }
    }

    pub fn parse(buffer: &[u8; 100]) -> SqliteResult<Self> {
        // let mut buffer = [0u8; 100];
        // reader.read_exact(&mut buffer)?;
        let magic: &[u8; 16] = &buffer[0..16].try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid magic string")
        })?;
        Self::magic_ok(magic)?;
        Ok(Self {
            magic_string: *magic,
            page_size: u16::from_be_bytes(
                buffer[16..18]
                    .try_into()
                    .map_err(|_| SqliteError::ParseHeaderField("page_size".into()))?,
            ),
            write_version: buffer[18],
            read_version: buffer[19],
            reserved_space: buffer[20],
            max_payload_fraction: buffer[21],
            min_payload_fraction: buffer[22],
            leaf_payload_fraction: buffer[23],
            file_change_counter: u32::from_be_bytes(
                buffer[24..28].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("file_change_counter".into())
                })?,
            ),
            database_size: u32::from_be_bytes(
                buffer[28..32].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("database_size".into())
                })?,
            ),
            first_freelist_trunk_page: u32::from_be_bytes(
                buffer[32..36].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("first_freelist_trunk_page".into())
                })?,
            ),
            total_freelist_pages: u32::from_be_bytes(
                buffer[36..40].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("total_freelist_pages".into())
                })?,
            ),
            schema_cookie: u32::from_be_bytes(
                buffer[40..44].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("schema_cookie".into())
                })?,
            ),
            schema_format_number: u32::from_be_bytes(
                buffer[44..48].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("schema_format_number".into())
                })?,
            ),
            default_page_cache_size: u32::from_be_bytes(
                buffer[48..52].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("default_page_cache_size".into())
                })?,
            ),
            largest_root_btree_page: u32::from_be_bytes(
                buffer[52..56].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("largest_root_btree_page".into())
                })?,
            ),
            text_encoding: u32::from_be_bytes(
                buffer[56..60].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("text_encoding".into())
                })?,
            ),
            user_version: u32::from_be_bytes(
                buffer[60..64].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("user_version".into())
                })?,
            ),
            incremental_vacuum_mode: u32::from_be_bytes(
                buffer[64..68].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("incremental_vacuum_mode".into())
                })?,
            ),
            application_id: u32::from_be_bytes(
                buffer[68..72].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("application_id".into())
                })?,
            ),
            reserved: buffer[72..92]
                .try_into()
                .map_err(|_| SqliteError::ParseHeaderField("reserved".into()))?,
            version_valid_for: u32::from_be_bytes(buffer[92..96].try_into().map_err(
                |_| SqliteError::ParseHeaderField("version_valid_for".into()),
            )?),
            sqlite_version_number: u32::from_be_bytes(
                buffer[96..100].try_into().map_err(|_| {
                    SqliteError::ParseHeaderField("sqlite_version_number".into())
                })?,
            ),
        })
    }

    pub fn page_size_ok(&self) -> SqliteResult<()> {
        if self.page_size == 1
            || (self.page_size.is_power_of_two()
                && self.page_size >= 512
                && self.page_size <= 32768)
        {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField("page_size".into()))
        }
    }

    pub fn file_format_write_version_ok(&self) -> SqliteResult<()> {
        if self.write_version == 1 || self.write_version == 2 {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField("write_version".into()))
        }
    }

    pub fn file_format_read_version_ok(&self) -> SqliteResult<()> {
        if self.read_version == 1 || self.read_version == 2 {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField("read_version".into()))
        }
    }

    pub fn reserved_space_ok(&self) -> SqliteResult<()> {
        if self.reserved_space <= 32 {
            let usable_size =
                u32::from(self.page_size) - u32::from(self.reserved_space);
            if usable_size >= 480 {
                Ok(())
            } else {
                Err(SqliteError::InvalidHeaderField("reserved_space".into()))
            }
        } else {
            Err(SqliteError::InvalidHeaderField("reserved_space".into()))
        }
    }

    pub fn payload_fractions_ok(&self) -> SqliteResult<()> {
        if self.max_payload_fraction == 64
            && self.min_payload_fraction == 32
            && self.leaf_payload_fraction == 32
        {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField("payload_fractions".into()))
        }
    }

    pub fn text_encoding_ok(&self) -> SqliteResult<()> {
        if self.text_encoding == 1 || self.text_encoding == 2 || self.text_encoding == 3
        {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField("text_encoding".into()))
        }
    }

    pub fn schema_format_number_ok(&self) -> SqliteResult<()> {
        if self.schema_format_number == 1
            || self.schema_format_number == 2
            || self.schema_format_number == 3
            || self.schema_format_number == 4
        {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField(
                "schema_format_number".into(),
            ))
        }
    }

    pub fn reserved_expansion_space_ok(&self) -> SqliteResult<()> {
        if self.reserved.iter().all(|&byte| byte == 0) {
            Ok(())
        } else {
            Err(SqliteError::InvalidHeaderField(
                "reserved_expansion_space".into(),
            ))
        }
    }

    pub fn is_ok(&self) -> SqliteResult<()> {
        self.page_size_ok()?;
        self.file_format_write_version_ok()?;
        self.file_format_read_version_ok()?;
        self.reserved_space_ok()?;
        self.payload_fractions_ok()?;
        self.text_encoding_ok()?;
        self.schema_format_number_ok()?;
        self.reserved_expansion_space_ok()?;
        Ok(())
    }
}
