//! tile-type module
//!
//! This is strongly influenced by the `TileInfo` struct from the `martin` crate.
//! The original version of this module was written and much more aligned with
//! the npm package `@mapbox/tiletype` and did not include `TileEncoding`.

use std::fmt::Display;

/// Tile format
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TileFormat {
    /// Unknown format
    Unknown,

    // ===================
    // VECTOR TILE FORMATS
    // ===================
    /// MVT Protocol Buffer format (AKA mvt)
    Pbf,

    /// MLT (Maplibre vector tile) future format
    Mlt,

    // =============
    // Image formats
    // =============
    /// GIF image
    Gif,

    /// JPEG image
    Jpg,

    /// PNG image
    Png,

    /// TIFF image
    Tiff,

    /// `WebP` image
    Webp,

    // ============
    // JSON FORMATS
    // ============
    /// JSON string
    Json,

    /// `GeoJSON` string
    GeoJson,
}

impl Display for TileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Pbf => "pbf",
            Self::Mlt => "mlt",
            Self::Json => "json",
            Self::GeoJson => "geojson",
            Self::Tiff => "tiff",
            Self::Unknown => "unknown",
        };
        write!(f, "{s}")
    }
}

impl TileFormat {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "png" => Self::Png,
            "webp" => Self::Webp,
            "pbf" | "mvt" => Self::Pbf,
            "gif" => Self::Gif,
            "jpg" | "jpeg" => Self::Jpg,
            "json" => Self::Json,
            "geojson" => Self::GeoJson,
            _ => None?,
        })
    }

    #[must_use]
    pub fn is_img(&self) -> bool {
        matches!(
            self,
            Self::Png | Self::Jpg | Self::Gif | Self::Webp | Self::Tiff
        )
    }

    #[must_use]
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Pbf | Self::Mlt => "application/x-protobuf",
            Self::Json => "application/json",
            Self::GeoJson => "application/geo+json",
            Self::Tiff => "image/tiff",
            Self::Unknown => "application/octet-stream",
        }
    }
}

/// Encoding of the tile data (based on maplibre/martin)
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TileEncoding {
    /// Data is not compressed, but it can be
    Uncompressed = 0b0000_0000,
    /// Data is compressed w/ internal encoding (e.g. jpg/png/webp)
    Internal = 0b0000_0001,
    /// Data is compressed w/ `gzip`
    Gzip = 0b0000_0010,
    /// Data is compressed w/ `zlib`
    Zlib = 0b0000_0100,
    /// Data is compressed w/ `brotli`
    Brotli = 0b0000_1000,
    /// Data is compressed w/ `zstd`
    Zstd = 0b0001_0000,
}

impl TileEncoding {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "none" => Self::Uncompressed,
            "gzip" | "gz" => Self::Gzip,
            "zlib" | "deflate" | "zz" => Self::Zlib,
            "brotli" | "br" => Self::Brotli,
            "zstd" | "zst" => Self::Zstd,
            "internal" | "png" | "jpg" | "jpeg" | "webp" | "gif" => Self::Internal,
            _ => None?,
        })
    }

    #[must_use]
    pub fn content_encoding(&self) -> Option<&'static str> {
        match self {
            Self::Internal | Self::Uncompressed => None,
            Self::Gzip => Some("gzip"),
            Self::Zlib => Some("deflate"),
            Self::Brotli => Some("br"),
            Self::Zstd => Some("zstd"),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uncompressed => "none",
            Self::Internal => "internal",
            Self::Gzip => "gzip",
            Self::Zlib => "zlib",
            Self::Brotli => "brotli",
            Self::Zstd => "zstd",
        }
    }
}

impl Display for TileEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.as_str();
        write!(f, "{s}")
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TileType {
    pub encoding: TileEncoding,
    pub format: TileFormat,
}

impl TileType {
    #[must_use]
    pub fn new(format: TileFormat, encoding: TileEncoding) -> Self {
        Self { encoding, format }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "geojson" => Self::new(TileFormat::GeoJson, TileEncoding::Uncompressed),
            "gif" => Self::new(TileFormat::Gif, TileEncoding::Internal),
            "jpg" | "jpeg" => Self::new(TileFormat::Jpg, TileEncoding::Internal),
            "json" => Self::new(TileFormat::Json, TileEncoding::Uncompressed),
            "mlt" => Self::new(TileFormat::Mlt, TileEncoding::Uncompressed),
            "pbf" | "mvt" => Self::new(TileFormat::Pbf, TileEncoding::Uncompressed),
            "pbf.gz" => Self::new(TileFormat::Pbf, TileEncoding::Gzip),
            "pbf.zlib" => Self::new(TileFormat::Pbf, TileEncoding::Zlib),
            "pbf.zst" => Self::new(TileFormat::Pbf, TileEncoding::Zstd),
            "png" => Self::new(TileFormat::Png, TileEncoding::Internal),
            "tiff" => Self::new(TileFormat::Tiff, TileEncoding::Uncompressed),
            "webp" => Self::new(TileFormat::Webp, TileEncoding::Internal),
            _ => None?,
        })
    }

    #[must_use]
    pub fn from_bytes(buffer: &[u8]) -> Self {
        if buffer.len() >= 8 {
            match buffer {
                v if v.starts_with(b"\x1f\x8b") => {
                    Self::new(TileFormat::Pbf, TileEncoding::Gzip)
                }
                v if zlib_magic_headers(v) => {
                    Self::new(TileFormat::Pbf, TileEncoding::Zlib)
                }
                v if zstd_magic_headers(v) => {
                    Self::new(TileFormat::Pbf, TileEncoding::Zstd)
                }
                v if v.starts_with(b"\x89PNG\r\n\x1a\n") => {
                    Self::new(TileFormat::Png, TileEncoding::Internal)
                }
                v if v.starts_with(b"\xff\xd8") => {
                    Self::new(TileFormat::Jpg, TileEncoding::Internal)
                }
                v if is_webp_buf(v) => {
                    Self::new(TileFormat::Webp, TileEncoding::Internal)
                }
                v if v.starts_with(b"GIF87a") || v.starts_with(b"GIF89a") => {
                    Self::new(TileFormat::Gif, TileEncoding::Internal)
                }
                v if v.starts_with(b"{") || v.starts_with(b"[") => {
                    Self::new(TileFormat::Json, TileEncoding::Uncompressed)
                }
                _ => Self::new(TileFormat::Unknown, TileEncoding::Uncompressed),
            }
        } else {
            Self::new(TileFormat::Unknown, TileEncoding::Uncompressed)
        }
    }

    #[must_use]
    pub fn content_type(&self) -> &'static str {
        self.format.content_type()
    }

    #[must_use]
    pub fn content_encoding(&self) -> Option<&'static str> {
        self.encoding.content_encoding()
    }

    #[must_use]
    pub fn headers_vec(&self) -> Vec<(&'static str, &'static str)> {
        if let Some(content_encoding) = self.content_encoding() {
            vec![
                ("Content-Type", self.content_type()),
                ("Content-Encoding", content_encoding),
            ]
        } else {
            vec![("Content-Type", self.content_type())]
        }
    }

    #[must_use]
    pub fn extension(&self) -> String {
        let fmt_ext = self.format.to_string();
        if self.format.is_img() {
            fmt_ext
        } else {
            match self.encoding {
                TileEncoding::Internal | TileEncoding::Uncompressed => fmt_ext,
                TileEncoding::Gzip => format!("{fmt_ext}.gz"),
                TileEncoding::Zlib => format!("{fmt_ext}.zlib"),
                TileEncoding::Brotli => format!("{fmt_ext}.br"),
                TileEncoding::Zstd => format!("{fmt_ext}.zst"),
            }
        }
    }
}

impl Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.format, self.encoding)
    }
}

///////////////////////////////////////////////////////////////////////////////
// legacy
///////////////////////////////////////////////////////////////////////////////

/// `TileType` or format of the tile data
pub enum TileTypeV1 {
    /// Unknown format
    Unknown = 0,

    /// GIF image
    Gif = 1,

    /// JPEG image
    Jpg = 2,

    /// JSON string
    Json = 3,

    /// Protocol Buffer format (AKA mvt)
    Pbf = 4,

    /// Protocol Buffer format (AKA mvt) compressed with gzip
    Pbfgz = 5,

    /// PNG image
    Png = 6,

    /// `WebP` image
    Webp = 7,
}
impl TileTypeV1 {
    #[must_use]
    pub fn headers(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            TileTypeV1::Png => vec![("Content-Type", "image/png")],
            TileTypeV1::Jpg => vec![("Content-Type", "image/jpeg")],
            TileTypeV1::Json => vec![("Content-Type", "application/json")],
            TileTypeV1::Gif => vec![("Content-Type", "image/gif")],
            TileTypeV1::Webp => vec![("Content-Type", "image/webp")],
            TileTypeV1::Pbf => vec![
                ("Content-Type", "application/x-protobuf"),
                ("Content-Encoding", "deflate"),
            ],
            TileTypeV1::Pbfgz => vec![
                ("Content-Type", "application/x-protobuf"),
                ("Content-Encoding", "gzip"),
            ],
            TileTypeV1::Unknown => vec![],
        }
    }
}

/// constant for unknown tile type
pub const TILETYPE_UNKNOWN: usize = 0;

/// constant for gif tile type
pub const TILETYPE_GIF: usize = 1;

/// constant for jpg tile type
pub const TILETYPE_JPG: usize = 2;

/// constant for json tile type
pub const TILETYPE_JSON: usize = 3;

/// constant for pbf tile type
pub const TILETYPE_PBF: usize = 4;

/// constant for pbfgz tile type
pub const TILETYPE_PBFGZ: usize = 5;

/// constant for png tile type
pub const TILETYPE_PNG: usize = 6;

/// constant for webp tile type
pub const TILETYPE_WEBP: usize = 7;

/// Return true if buffer starts with zlib magic headers
/// 78 01 - No Compression/low
/// 78 5E - Fast Compression
/// 78 9C - Default Compression
/// 78 DA - Best Compression
#[must_use]
#[inline]
pub fn zlib_magic_headers(buffer: &[u8]) -> bool {
    buffer.starts_with(
        b"\x78\x01", // No Compression/low
    ) || buffer.starts_with(
        b"\x78\x5E", // Fast Compression
    ) || buffer.starts_with(
        b"\x78\x9C", // Default Compression
    ) || buffer.starts_with(
        b"\x78\xDA", // Best Compression
    )
}

/// zstd magic headers
/// 28 B5 2F FD
#[must_use]
#[inline]
pub fn zstd_magic_headers(buffer: &[u8]) -> bool {
    buffer.starts_with(b"\x28\xB5\x2F\xFD")
}

#[must_use]
#[inline]
pub fn is_webp_buf(data: &[u8]) -> bool {
    data.starts_with(b"RIFF") && data.len() > 8 && data[8..].starts_with(b"WEBP")
}

/// Return type of the tile data from a buffer
#[must_use]
pub fn tiletype(buffer: &[u8]) -> TileType {
    // if buffer.len() >= 8 {
    //     match buffer {
    //         v if v.starts_with(b"\x1f\x8b") => return TileType::Pbfgz,
    //         v if zlib_magic_headers(v) => return TileType::Pbf,
    //         v if v.starts_with(b"\x89PNG\r\n\x1a\n") => return TileType::Png,
    //         v if v.starts_with(b"\xff\xd8") => return TileType::Jpg,
    //         v if is_webp_buf(v) => return TileType::Webp,
    //         v if v.starts_with(b"GIF87a") || v.starts_with(b"GIF89a") => {
    //             return TileType::Gif;
    //         }
    //         v if v.starts_with(b"{") || v.starts_with(b"[") => return TileType::Json,
    //         _ => {}
    //     }
    // }
    // TileType::Unknown
    TileType::from_bytes(buffer)
}

/// Return the tile type as a constant
#[must_use]
pub fn enum2const(tiletype: &TileTypeV1) -> usize {
    match tiletype {
        TileTypeV1::Unknown => TILETYPE_UNKNOWN,
        TileTypeV1::Gif => TILETYPE_GIF,
        TileTypeV1::Jpg => TILETYPE_JPG,
        TileTypeV1::Json => TILETYPE_JSON,
        TileTypeV1::Pbf => TILETYPE_PBF,
        TileTypeV1::Pbfgz => TILETYPE_PBFGZ,
        TileTypeV1::Png => TILETYPE_PNG,
        TileTypeV1::Webp => TILETYPE_WEBP,
    }
}

/// Return the tile type as an enum
#[must_use]
pub fn const2enum(tiletype: usize) -> TileTypeV1 {
    match tiletype {
        TILETYPE_GIF => TileTypeV1::Gif,
        TILETYPE_JPG => TileTypeV1::Jpg,
        TILETYPE_JSON => TileTypeV1::Json,
        TILETYPE_PBF => TileTypeV1::Pbf,
        TILETYPE_PBFGZ => TileTypeV1::Pbfgz,
        TILETYPE_PNG => TileTypeV1::Png,
        TILETYPE_WEBP => TileTypeV1::Webp,
        _ => TileTypeV1::Unknown,
    }
}

/// Return vector of http headers for a tile type
#[must_use]
pub fn headers(tiletype: &TileTypeV1) -> Vec<(&'static str, &'static str)> {
    tiletype.headers()
}

/// Return vector of http headers for a tile type from a tile's buffer
#[must_use]
pub fn blob2headers(b: &[u8]) -> Vec<(&'static str, &'static str)> {
    tiletype(b).headers_vec()
}

/// Return the tile type as a string
#[must_use]
pub fn tiletype_str(buffer: &[u8]) -> String {
    let tiletype = tiletype(buffer);
    tiletype.extension()
}
