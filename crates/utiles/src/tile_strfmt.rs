use std::fmt::{Display, Formatter};
use utiles_core::TileLike;

#[derive(Debug, PartialEq)]
pub enum FormatTokens {
    X,
    Y,
    Z,
    ZxyFslash,
    Quadkey,
    JsonObj,
    JsonArr,
}

impl Display for FormatTokens {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            FormatTokens::X => "{x}",
            FormatTokens::Y => "{y}",
            FormatTokens::Z => "{z}",
            FormatTokens::ZxyFslash => "{z}/{x}/{y}",
            FormatTokens::Quadkey => "{quadkey}",
            FormatTokens::JsonObj => "{json_obj}",
            FormatTokens::JsonArr => "{json_arr}",
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
            FormatTokens::ZxyFslash => "{z}/{x}/{y}",
            FormatTokens::Quadkey => "{quadkey}",
            FormatTokens::JsonObj => "{json_obj}",
            FormatTokens::JsonArr => "{json_arr}",
        }
    }
}

impl From<&str> for FormatParts {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "x" => FormatParts::Token(FormatTokens::X),
            "y" | "yup" | "-y" => FormatParts::Token(FormatTokens::Y),
            "z" => FormatParts::Token(FormatTokens::Z),
            "zxy" => FormatParts::Token(FormatTokens::ZxyFslash),
            "quadkey" | "qk" => FormatParts::Token(FormatTokens::Quadkey),
            "json" | "json_arr" => FormatParts::Token(FormatTokens::JsonArr),
            "json_obj" => FormatParts::Token(FormatTokens::JsonObj),
            _ => FormatParts::Str(s.to_string()),
        }
    }
    // match token.to_lowercase().as_str() {
    // "x" => tokens.push(FormatParts::Token(FormatTokens::X)),
    // "y" | "yup" | "-y" => {
    // tokens.push(FormatParts::Token(FormatTokens::Y))
    // }
    // "z" => tokens.push(FormatParts::Token(FormatTokens::Z)),
    // "zxy" => tokens.push(FormatParts::Token(FormatTokens::ZxyFslash)),
    // "quadkey" | "qk" => {
    // tokens.push(FormatParts::Token(FormatTokens::Quadkey))
    // }
    // "json" | "json_arr" => {
    // tokens.push(FormatParts::Token(FormatTokens::JsonArr))
    // }
    // "json_obj" => {
    // tokens.push(FormatParts::Token(FormatTokens::JsonObj))
    // }
    // _ => tokens.push(FormatParts::Str(token.clone())),
    // }
}

impl From<&FormatTokens> for String {
    fn from(t: &FormatTokens) -> Self {
        match t {
            FormatTokens::X => "{x}".to_string(),
            FormatTokens::Y => "{y}".to_string(),
            FormatTokens::Z => "{z}".to_string(),
            FormatTokens::ZxyFslash => "{z}/{x}/{y}".to_string(),
            FormatTokens::Quadkey => "{quadkey}".to_string(),
            FormatTokens::JsonObj => "{json_obj}".to_string(),
            FormatTokens::JsonArr => "{json_arr}".to_string(),
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

pub struct TileStringFormatter {
    fmt: String,
    tokens: Vec<FormatParts>,
    n_tokens: usize,
}

impl TileStringFormatter {
    #[must_use]
    pub fn new(fmt: &str) -> Self {
        let (tokens, n_tokens) = TileStringFormatter::parse(fmt);
        let fmt_str = tokens.iter().map(String::from).collect::<String>();

        Self {
            fmt: fmt_str,
            tokens,
            n_tokens,
        }
    }

    fn parse(fmt_string: &str) -> (Vec<FormatParts>, usize) {
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

    pub fn format<T: TileLike>(&self, tile: &T) -> String {
        let mut out = String::new();
        for part in &self.tokens {
            match part {
                FormatParts::Str(s) => out.push_str(s),
                FormatParts::Token(t) => match t {
                    FormatTokens::X => out.push_str(&tile.x().to_string()),
                    FormatTokens::Y => out.push_str(&tile.y().to_string()),
                    FormatTokens::Z => out.push_str(&tile.z().to_string()),
                    FormatTokens::ZxyFslash => out.push_str(&tile.zxy_str_fslash()),
                    FormatTokens::Quadkey => out.push_str(&tile.quadkey()),
                    FormatTokens::JsonArr => out.push_str(&tile.json_arr()),
                    FormatTokens::JsonObj => out.push_str(&tile.json_obj()),
                },
            }
        }
        out
    }

    #[must_use]
    pub fn has_token(&self) -> bool {
        self.n_tokens > 0
    }

    #[must_use]
    pub fn fmt(&self) -> &str {
        &self.fmt
    }
}

impl Default for TileStringFormatter {
    fn default() -> Self {
        Self {
            fmt: "{json}".to_string(),
            tokens: vec![FormatParts::Token(FormatTokens::ZxyFslash)],
            n_tokens: 1,
        }
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
        assert_eq!(f.n_tokens, 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.format(&tile), "3/1/2");
    }

    #[test]
    fn test_formatter_zxy_fslash() {
        let fmt = "{zxy}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens, 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.format(&tile), "3/1/2");
    }

    #[test]
    fn test_formatter_quadkey() {
        let fmt = "{quadkey}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens, 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.format(&tile), "021");
    }

    #[test]
    fn test_formatter_json_arr() {
        let fmt = "{json_arr}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens, 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.format(&tile), "[1, 2, 3]");
    }

    #[test]
    fn test_formatter_json_obj() {
        let fmt = "{json_obj}";
        let f = TileStringFormatter::new(fmt);
        assert_eq!(f.n_tokens, 1);
        let tile = Tile::new(1, 2, 3);
        assert_eq!(f.format(&tile), "{\"x\":1, \"y\":2, \"z\":3}");
    }

    #[test]
    fn test_formatter_combined() {
        let fmt = "tiles/{z}/{x}/{y}";
        let f = TileStringFormatter::new(fmt);
        let tile = Tile::new(1, 2, 3);
        println!("{:?}", f.tokens);

        assert_eq!(
            f.tokens,
            vec![
                FormatParts::Str("tiles/".to_string()),
                FormatParts::Token(FormatTokens::ZxyFslash),
            ]
        );
        assert_eq!(f.n_tokens, 1);
        assert_eq!(f.format(&tile), "tiles/3/1/2");
    }
}
