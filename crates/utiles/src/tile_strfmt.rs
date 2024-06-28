//! Tile string formatting
use std::fmt::{Display, Formatter};
use utiles_core::bbox::WebBBox;
use utiles_core::{BBox, TileLike};

#[derive(Debug, PartialEq)]
pub enum FormatTokens {
    X,
    Y,
    Z,
    Yup,
    ZxyFslash,
    Quadkey,
    PmtileId,
    JsonObj,
    JsonArr,
    GeoBBox,
    Projwin,
    BBoxWeb,
    ProjwinWeb,
}

impl Display for FormatTokens {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            FormatTokens::X => "{x}",
            FormatTokens::Y => "{y}",
            FormatTokens::Z => "{z}",
            FormatTokens::Yup => "{-y}",
            FormatTokens::ZxyFslash => "{z}/{x}/{y}",
            FormatTokens::Quadkey => "{quadkey}",
            FormatTokens::PmtileId => "{pmtileid}",
            FormatTokens::JsonObj => "{json_obj}",
            FormatTokens::JsonArr => "{json_arr}",
            FormatTokens::GeoBBox => "{bbox}",
            FormatTokens::Projwin => "{projwin}",
            FormatTokens::BBoxWeb => "{bbox_web}",
            FormatTokens::ProjwinWeb => "{projwin_web}",
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum FormatParts {
    Str(String),
    Token(FormatTokens),
}

impl From<FormatTokens> for &'static str {
    fn from(t: FormatTokens) -> Self {
        match t {
            FormatTokens::X => "{x}",
            FormatTokens::Y => "{y}",
            FormatTokens::Z => "{z}",
            FormatTokens::Yup => "{-y}",
            FormatTokens::ZxyFslash => "{z}/{x}/{y}",
            FormatTokens::Quadkey => "{quadkey}",
            FormatTokens::PmtileId => "{pmtileid}",
            FormatTokens::JsonObj => "{json_obj}",
            FormatTokens::JsonArr => "{json_arr}",
            FormatTokens::GeoBBox => "{bbox}",
            FormatTokens::Projwin => "{projwin}",
            FormatTokens::BBoxWeb => "{bbox_web}",
            FormatTokens::ProjwinWeb => "{projwin_web}",
        }
    }
}

impl From<&str> for FormatParts {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "x" => FormatParts::Token(FormatTokens::X),
            "y" => FormatParts::Token(FormatTokens::Y),
            "z" => FormatParts::Token(FormatTokens::Z),
            "yup" | "-y" => FormatParts::Token(FormatTokens::Yup),
            "zxy" => FormatParts::Token(FormatTokens::ZxyFslash),
            "quadkey" | "qk" => FormatParts::Token(FormatTokens::Quadkey),
            "pmtileid" | "pmid" => FormatParts::Token(FormatTokens::PmtileId),
            "json" | "json_arr" => FormatParts::Token(FormatTokens::JsonArr),
            "json_obj" | "obj" => FormatParts::Token(FormatTokens::JsonObj),
            "bbox" => FormatParts::Token(FormatTokens::GeoBBox),
            "projwin" => FormatParts::Token(FormatTokens::Projwin),
            "bbox_web" => FormatParts::Token(FormatTokens::BBoxWeb),
            "projwin_web" => FormatParts::Token(FormatTokens::ProjwinWeb),

            _ => FormatParts::Str(s.to_string()),
        }
    }
}

impl From<&FormatTokens> for String {
    fn from(t: &FormatTokens) -> Self {
        match t {
            FormatTokens::X => "{x}".to_string(),
            FormatTokens::Y => "{y}".to_string(),
            FormatTokens::Z => "{z}".to_string(),
            FormatTokens::Yup => "{-y}".to_string(),
            FormatTokens::ZxyFslash => "{z}/{x}/{y}".to_string(),
            FormatTokens::Quadkey => "{quadkey}".to_string(),
            FormatTokens::PmtileId => "{pmtileid}".to_string(),
            FormatTokens::JsonObj => "{json_obj}".to_string(),
            FormatTokens::JsonArr => "{json_arr}".to_string(),
            FormatTokens::GeoBBox => "{bbox}".to_string(),
            FormatTokens::Projwin => "{projwin}".to_string(),
            FormatTokens::BBoxWeb => "{bbox_web}".to_string(),
            FormatTokens::ProjwinWeb => "{projwin_web}".to_string(),
        }
    }
}

impl From<&FormatParts> for String {
    fn from(p: &FormatParts) -> Self {
        match p {
            FormatParts::Str(s) => s.to_string(),
            FormatParts::Token(t) => String::from(t),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileStringFormat {
    fmtstr: String,
    tokens: Vec<FormatParts>,
    n_tokens: usize,
}

impl Default for TileStringFormat {
    fn default() -> Self {
        Self {
            fmtstr: "{json_arr}".to_string(),
            tokens: vec![FormatParts::Token(FormatTokens::ZxyFslash)],
            n_tokens: 1,
        }
    }
}

impl TileStringFormat {
    pub fn new(fmt: &str) -> Self {
        let (tokens, n_tokens) = TileStringFormat::parse(fmt);
        let fmt_str = tokens.iter().map(String::from).collect::<String>();
        Self {
            fmtstr: fmt_str,
            tokens,
            n_tokens,
        }
    }

    fn parse(fmt_string: &str) -> (Vec<FormatParts>, usize) {
        // check if the fmt string is just a token
        let fmt_string = fmt_string.trim();
        // else we do the full parse
        let fmt = fmt_string
            .replace("{z}/{x}/{y}", "{zxy}")
            .replace("{x}/{y}/{z}", "{xyz}");
        let mut tokens = Vec::new();
        let mut token = String::new();
        for c in fmt.chars() {
            if c == '{' {
                if !token.is_empty() {
                    tokens.push(FormatParts::Str(token.clone()));
                    token.clear();
                }
                continue;
            }
            if c == '}' {
                if !token.is_empty() {
                    tokens.push(FormatParts::from(token.as_str()));
                    token.clear();
                }
                continue;
            }
            token.push(c);
        }
        if !token.is_empty() {
            tokens.push(FormatParts::Str(token));
        }
        let n_tokens = tokens
            .iter()
            .filter(|t| !matches!(t, FormatParts::Str(_)))
            .count();
        (tokens, n_tokens)
    }
}

pub struct TileStringFormatter {
    tile_fmt: TileStringFormat,
    parts: Vec<Part>,
}

enum Part {
    Static(&'static str),
    Dynamic(fn(&dyn TileLike) -> String),
}

impl TileStringFormatter {
    #[must_use]
    pub fn new(fmt: &str) -> Self {
        let tile_fmt = TileStringFormat::new(fmt);
        let parts = TileStringFormatter::parse_parts(&tile_fmt);
        Self { tile_fmt, parts }
    }

    fn parse_parts(tile_fmt: &TileStringFormat) -> Vec<Part> {
        let mut parts = Vec::new();
        for token in &tile_fmt.tokens {
            match token {
                FormatParts::Str(s) => {
                    parts.push(Part::Static(Box::leak(s.clone().into_boxed_str())));
                }
                FormatParts::Token(t) => match t {
                    FormatTokens::X => {
                        parts.push(Part::Dynamic(|tile| tile.x().to_string()));
                    }
                    FormatTokens::Y => {
                        parts.push(Part::Dynamic(|tile| tile.y().to_string()));
                    }
                    FormatTokens::Yup => {
                        parts.push(Part::Dynamic(|tile| (tile.yup()).to_string()));
                    }
                    FormatTokens::Z => {
                        parts.push(Part::Dynamic(|tile| tile.z().to_string()));
                    }
                    FormatTokens::ZxyFslash => {
                        parts.push(Part::Dynamic(|tile| tile.zxy_str_fslash()));
                    }
                    FormatTokens::Quadkey => {
                        parts.push(Part::Dynamic(|tile| tile.quadkey()));
                    }
                    FormatTokens::PmtileId => {
                        parts.push(Part::Dynamic(|tile| tile.pmtileid().to_string()));
                    }
                    FormatTokens::JsonArr => {
                        parts.push(Part::Dynamic(|tile| tile.json_arr()));
                    }
                    FormatTokens::JsonObj => {
                        parts.push(Part::Dynamic(|tile| tile.json_obj()));
                    }
                    FormatTokens::GeoBBox => {
                        parts.push(Part::Dynamic(|tile| {
                            let b: BBox = tile.bbox().into();
                            b.json_arr()
                        }));
                    }
                    FormatTokens::Projwin => {
                        parts.push(Part::Dynamic(|tile| {
                            let b: BBox = tile.bbox().into();
                            b.projwin_str()
                        }));
                    }

                    FormatTokens::ProjwinWeb => {
                        parts.push(Part::Dynamic(|tile| {
                            let b: WebBBox = tile.webbbox();
                            b.projwin_str()
                        }));
                    }
                    FormatTokens::BBoxWeb => {
                        parts.push(Part::Dynamic(|tile| {
                            let b: WebBBox = tile.webbbox();
                            b.json_arr()
                        }));
                    }
                },
            }
        }
        parts
    }

    #[must_use]
    pub fn tokens(&self) -> &Vec<FormatParts> {
        &self.tile_fmt.tokens
    }

    #[must_use]
    pub fn n_tokens(&self) -> usize {
        self.tile_fmt.n_tokens
    }

    pub fn fmt_tile_custom<T: TileLike>(&self, tile: &T) -> String {
        let mut out = String::with_capacity(self.tile_fmt.fmtstr.len() * 2); // Assuming average length doubling due to replacements
        for part in &self.parts {
            match part {
                Part::Static(s) => out.push_str(s),
                Part::Dynamic(f) => out.push_str(&f(tile)),
            }
        }
        out
    }

    pub fn fmt_tile<T: TileLike>(&self, tile: &T) -> String {
        match self.tile_fmt.fmtstr.as_str() {
            "{json_arr}" => tile.json_arr(),
            "{json_obj}" => tile.json_obj(),
            "{quadkey}" => tile.quadkey(),
            "{zxy}" => tile.zxy_str_fslash(),
            _ => self.fmt_tile_custom(tile),
        }
    }

    #[must_use]
    pub fn has_token(&self) -> bool {
        self.tile_fmt.n_tokens > 0
    }

    #[must_use]
    pub fn fmt(&self) -> &str {
        &self.tile_fmt.fmtstr
    }
}

impl Default for TileStringFormatter {
    fn default() -> Self {
        Self::new("{json_arr}")
    }
}

#[cfg(test)]
mod tests {
    use utiles_core::Tile;

    use super::*;

    #[test]
    fn test_formatter_zxy() {
        let fmt = "{z}/{x}/{y}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens(), 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.fmt_tile(&tile), "3/1/2");
    }

    #[test]
    fn test_formatter_zxy_fslash() {
        let fmt = "{zxy}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens(), 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.fmt_tile(&tile), "3/1/2");
    }

    #[test]
    fn test_formatter_quadkey() {
        let fmt = "{quadkey}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens(), 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.fmt_tile(&tile), "021");
    }

    #[test]
    fn test_formatter_json_arr() {
        let fmt = "{json_arr}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens(), 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.fmt_tile(&tile), "[1, 2, 3]");
    }

    #[test]
    fn test_formatter_json_obj() {
        let fmt = "{json_obj}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens(), 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.fmt_tile(&tile), "{\"x\":1, \"y\":2, \"z\":3}");
    }

    #[test]
    fn test_formatter_combined() {
        let fmt = "tiles/{z}/{x}/{y}";
        let f = TileStringFormatter::new(fmt);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(
            *f.tokens(),
            vec![
                FormatParts::Str("tiles/".to_string()),
                FormatParts::Token(FormatTokens::ZxyFslash),
            ]
        );
        assert_eq!(f.n_tokens(), 1);
        assert_eq!(f.fmt_tile(&tile), "tiles/3/1/2");
    }
}
