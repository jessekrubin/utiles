use crate::Tile;

#[must_use]
pub fn parse_textiles(s: &str) -> Vec<Tile> {
    s.split('\n')
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| Tile::from_json(line).ok())
        .collect()
}
