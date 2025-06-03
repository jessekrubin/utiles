use crate::tile_tuple::TileTuple;
use crate::{Tile, UtilesCoreError};
use serde_json::{Map, Value};
use std::str::FromStr;

// =============================================================================
// From ... for TILE
// =============================================================================

impl From<(u8, u32, u32)> for Tile {
    fn from(xyz: (u8, u32, u32)) -> Self {
        Tile {
            z: xyz.0,
            x: xyz.1,
            y: xyz.2,
        }
    }
}

impl From<(u32, u32, u8)> for Tile {
    fn from(xyz: (u32, u32, u8)) -> Self {
        Tile {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl From<TileTuple> for Tile {
    fn from(xyz: TileTuple) -> Self {
        Tile {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl From<Tile> for (u32, u32, u8) {
    fn from(tile: Tile) -> Self {
        (tile.x, tile.y, tile.z)
    }
}

impl TryFrom<(u32, u32, u32)> for Tile {
    type Error = UtilesCoreError;

    fn try_from(xyz: (u32, u32, u32)) -> Result<Self, Self::Error> {
        let z = u8::try_from(xyz.0)?;
        if z > 30 {
            return Err(UtilesCoreError::InvalidTile(format!(
                "Invalid tile zoom level: {}. Maximum is 30.",
                xyz.2
            )));
        }
        let max_xy = (1 << z) - 1;
        if xyz.1 > max_xy || xyz.2 > max_xy {
            Err(UtilesCoreError::InvalidTile(format!(
                "Invalid tile coordinates: x: {}, y: {}, z: {}",
                xyz.0, xyz.1, xyz.2
            )))
        } else {
            Ok(Tile {
                x: xyz.0,
                y: xyz.1,
                z,
            })
        }
    }
}

impl TryFrom<&str> for Tile {
    type Error = UtilesCoreError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let res = Tile::from_str(s);
        match res {
            Ok(tile) => Ok(tile),
            Err(e) => Err(UtilesCoreError::TileParseError(e.to_string())),
        }
    }
}

impl TryFrom<Value> for Tile {
    type Error = UtilesCoreError;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        Tile::try_from(&val)
    }
}

impl TryFrom<(u64, u64, u64)> for Tile {
    type Error = UtilesCoreError;

    fn try_from(tuple: (u64, u64, u64)) -> Result<Self, Self::Error> {
        let x = u32::try_from(tuple.0).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        let y = u32::try_from(tuple.1).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        let z = u8::try_from(tuple.2).map_err(|_| {
            UtilesCoreError::InvalidTile(format!(
                "({},{},{})",
                tuple.0, tuple.1, tuple.2
            ))
        })?;
        Ok(Tile::new(x, y, z))
    }
}

impl TryFrom<&Vec<Value>> for Tile {
    type Error = UtilesCoreError;

    fn try_from(arr: &Vec<Value>) -> Result<Self, Self::Error> {
        if arr.len() < 3 {
            Err(UtilesCoreError::InvalidJson(
                serde_json::to_string(&arr).unwrap_or_default(),
            ))
        } else {
            let x = arr[0].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            let y = arr[1].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            let z = arr[2].as_u64().ok_or_else(|| {
                UtilesCoreError::InvalidJson(
                    serde_json::to_string(&arr).unwrap_or_default(),
                )
            })?;
            Tile::try_from((x, y, z))
        }
    }
}

impl TryFrom<&Value> for Tile {
    type Error = UtilesCoreError;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::Array(v) => {
                let t = Tile::try_from(v)?;
                Ok(t)
            }
            Value::Object(v) => {
                if v.contains_key("x") && v.contains_key("y") && v.contains_key("z") {
                    let x = v["x"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    let y = v["y"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    let z = v["z"].as_u64().ok_or_else(|| {
                        UtilesCoreError::InvalidJson(
                            serde_json::to_string(&v)
                                .expect("Invalid json object for Tile from Value"),
                        )
                    })?;
                    Tile::try_from((x, y, z))
                } else if v.contains_key("tile")
                    && v["tile"].is_array()
                    && v["tile"]
                        .as_array()
                        .expect("Unable to get tile array from Value")
                        .len()
                        == 3
                {
                    let tuple = serde_json::from_value::<TileTuple>(v["tile"].clone())?;
                    Ok(Tile::from(tuple))
                } else {
                    Err(UtilesCoreError::InvalidJson(
                        serde_json::to_string(&v)
                            .expect("Invalid json object for Tile from Value"),
                    ))
                }
            }
            _ => Err(UtilesCoreError::InvalidJson(val.to_string())),
        }
    }
}

impl TryFrom<&Map<String, Value>> for Tile {
    type Error = UtilesCoreError;

    fn try_from(map: &Map<String, Value>) -> Result<Self, Self::Error> {
        let x = u32::try_from(map["x"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        let y = u32::try_from(map["y"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        let z = u8::try_from(map["z"].as_u64().ok_or_else(|| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?)
        .map_err(|_| {
            UtilesCoreError::InvalidJson(
                serde_json::to_string(&map).unwrap_or_default(),
            )
        })?;
        Ok(Tile::new(x, y, z))
    }
}
