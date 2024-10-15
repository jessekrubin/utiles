use crate::Tile;
use std::str::FromStr;

pub fn parse_textiles(s: &str) -> Vec<Tile> {
    let parsed: Vec<Tile> = s
        .split("\n")
        .map(Tile::from_str)
        .flatten()
        .collect::<Vec<Tile>>();
    parsed
}
