pub enum TileType {
    Unknown = 0,
    Gif = 1,
    Jpg = 2,
    Pbf = 3,
    Pbfgz = 4,
    Png = 5,
    Webp = 6,
}

pub const TILETYPE_UNKNOWN: usize = 0;
pub const TILETYPE_GIF: usize = 1;
pub const TILETYPE_JPG: usize = 2;
pub const TILETYPE_PBF: usize = 3;
pub const TILETYPE_PBFGZ: usize = 4;
pub const TILETYPE_PNG: usize = 5;
pub const TILETYPE_WEBP: usize = 6;

pub fn tiletype(buffer: &[u8]) -> TileType {
    if buffer.len() >= 8 {
        if buffer[0] == 0x89
            && buffer[1] == 0x50
            && buffer[2] == 0x4e
            && buffer[3] == 0x47
            && buffer[4] == 0x0d
            && buffer[5] == 0x0a
            && buffer[6] == 0x1a
            && buffer[7] == 0x0a
        {
            return TileType::Png;
        } else if buffer[0] == 0xff
            && buffer[1] == 0xd8
            && buffer[buffer.len() - 2] == 0xff
            && buffer[buffer.len() - 1] == 0xd9
        {
            return TileType::Jpg;
        } else if buffer[0] == 0x47
            && buffer[1] == 0x49
            && buffer[2] == 0x46
            && buffer[3] == 0x38
            && (buffer[4] == 0x39 || buffer[4] == 0x37)
            && buffer[5] == 0x61
        {
            return TileType::Gif;
        } else if buffer[0] == 0x52
            && buffer[1] == 0x49
            && buffer[2] == 0x46
            && buffer[3] == 0x46
            && buffer[8] == 0x57
            && buffer[9] == 0x45
            && buffer[10] == 0x42
            && buffer[11] == 0x50
        {
            return TileType::Webp;
        } else if buffer[0] == 0x78 && buffer[1] == 0x9c {
            return TileType::Pbf;
        } else if buffer[0] == 0x1f && buffer[1] == 0x8b {
            return TileType::Pbfgz;
        }
    }
    TileType::Unknown
}

pub fn enum2const(tiletype: TileType) -> usize {
    match tiletype {
        TileType::Unknown => TILETYPE_UNKNOWN,
        TileType::Gif => TILETYPE_GIF,
        TileType::Jpg => TILETYPE_JPG,
        TileType::Pbf => TILETYPE_PBF,
        TileType::Pbfgz => TILETYPE_PBFGZ,
        TileType::Png => TILETYPE_PNG,
        TileType::Webp => TILETYPE_WEBP,
    }
}

pub fn const2enum(tiletype: usize) -> TileType {
    match tiletype {
        TILETYPE_UNKNOWN => TileType::Unknown,
        TILETYPE_GIF => TileType::Gif,
        TILETYPE_JPG => TileType::Jpg,
        TILETYPE_PBF => TileType::Pbf,
        TILETYPE_PBFGZ => TileType::Pbfgz,
        TILETYPE_PNG => TileType::Png,
        TILETYPE_WEBP => TileType::Webp,
        _ => TileType::Unknown,
    }
}

pub fn headers(tiletype: TileType) -> Vec<(&'static str, &'static str)> {
    match tiletype {
        TileType::Png => vec![("Content-Type", "image/png")],
        TileType::Jpg => vec![("Content-Type", "image/jpeg")],
        TileType::Gif => vec![("Content-Type", "image/gif")],
        TileType::Webp => vec![("Content-Type", "image/webp")],
        TileType::Pbf => vec![
            ("Content-Type", "application/x-protobuf"),
            ("Content-Encoding", "gzip"),
        ],
        TileType::Pbfgz => vec![
            ("Content-Type", "application/x-protobuf"),
            ("Content-Encoding", "gzip"),
        ],
        _ => vec![],
    }
}

pub fn tiletype_str(buffer: &[u8]) -> String {
    let tiletype = tiletype(buffer);
    match tiletype {
        TileType::Unknown => "unknown".to_string(),
        TileType::Gif => "gif".to_string(),
        TileType::Jpg => "jpg".to_string(),
        TileType::Pbf => "pbf".to_string(),
        TileType::Pbfgz => "pbfgz".to_string(),
        TileType::Png => "png".to_string(),
        TileType::Webp => "webp".to_string(),
    }
}
