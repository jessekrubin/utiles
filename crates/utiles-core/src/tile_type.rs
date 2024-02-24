//! TileType module (needs work)

/// TileType or format of the tile data
pub enum TileType {
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

    /// WebP image
    Webp = 7,
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

/// Return type of the tile data from a buffer
#[must_use]
pub fn tiletype(buffer: &[u8]) -> TileType {
    if buffer.len() >= 8 {
        match buffer {
            v if v.starts_with(b"\x89PNG\r\n\x1a\n") => return TileType::Png,
            v if v.starts_with(b"\xff\xd8") => return TileType::Jpg,
            v if v.starts_with(b"GIF87a") || v.starts_with(b"GIF89a") => {
                return TileType::Gif
            }
            v if v.starts_with(b"RIFF") && &v[8..12] == b"WEBP" => {
                return TileType::Webp
            }
            v if v.starts_with(b"\x1f\x8b") => return TileType::Pbfgz,
            v if v.starts_with(b"\x78\x9c") => return TileType::Pbf,
            v if v.starts_with(b"{") || v.starts_with(b"[") => return TileType::Json,
            _ => {}
        }
    }
    TileType::Unknown
}

/// Return the tile type as a constant
#[must_use]
pub fn enum2const(tiletype: TileType) -> usize {
    match tiletype {
        TileType::Unknown => TILETYPE_UNKNOWN,
        TileType::Gif => TILETYPE_GIF,
        TileType::Jpg => TILETYPE_JPG,
        TileType::Json => TILETYPE_JSON,
        TileType::Pbf => TILETYPE_PBF,
        TileType::Pbfgz => TILETYPE_PBFGZ,
        TileType::Png => TILETYPE_PNG,
        TileType::Webp => TILETYPE_WEBP,
    }
}

/// Return the tile type as an enum
#[must_use]
pub fn const2enum(tiletype: usize) -> TileType {
    match tiletype {
        TILETYPE_GIF => TileType::Gif,
        TILETYPE_JPG => TileType::Jpg,
        TILETYPE_JSON => TileType::Json,
        TILETYPE_PBF => TileType::Pbf,
        TILETYPE_PBFGZ => TileType::Pbfgz,
        TILETYPE_PNG => TileType::Png,
        TILETYPE_WEBP => TileType::Webp,
        _ => TileType::Unknown,
    }
}

/// Return vector of http headers for a tile type
#[must_use]
pub fn headers(tiletype: &TileType) -> Vec<(&'static str, &'static str)> {
    match tiletype {
        TileType::Png => vec![("Content-Type", "image/png")],
        TileType::Jpg => vec![("Content-Type", "image/jpeg")],
        TileType::Json => vec![("Content-Type", "application/json")],
        TileType::Gif => vec![("Content-Type", "image/gif")],
        TileType::Webp => vec![("Content-Type", "image/webp")],
        TileType::Pbf | TileType::Pbfgz => vec![
            ("Content-Type", "application/x-protobuf"),
            ("Content-Encoding", "gzip"),
        ],
        _ => vec![],
    }
}

/// Return vector of http headers for a tile type from a tile's buffer
#[must_use]
pub fn blob2headers(b: &[u8]) -> Vec<(&'static str, &'static str)> {
    let tiletype = tiletype(b);
    headers(&tiletype)
}

/// Return the tile type as a string
#[must_use]
pub fn tiletype_str(buffer: &[u8]) -> String {
    let tiletype = tiletype(buffer);
    match tiletype {
        TileType::Unknown => "unknown".to_string(),
        TileType::Gif => "gif".to_string(),
        TileType::Jpg => "jpg".to_string(),
        TileType::Json => "json".to_string(),
        TileType::Pbf => "pbf".to_string(),
        TileType::Pbfgz => "pbfgz".to_string(),
        TileType::Png => "png".to_string(),
        TileType::Webp => "webp".to_string(),
    }
}
