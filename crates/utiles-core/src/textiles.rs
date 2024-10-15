use crate::Tile;
use std::str::FromStr;

pub fn parse_textiles(s: &str) -> Vec<Tile> {
    let parsed: Vec<Tile> = s
        .split('\n')
        .flat_map(Tile::from_str)
        .collect::<Vec<Tile>>();
    parsed
}
