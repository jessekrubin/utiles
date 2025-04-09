use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Value as JSONValue, Value};
use tilejson::{tilejson, Bounds, Center, TileJSON};

use utiles_core::geostats::TileStats;

use crate::errors::UtilesResult;
use crate::mbt::MbtMetadataRow;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TerrainRgbo {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub o: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Terrain {
    Rgbo(TerrainRgbo),

    // 'mapbox' string
    #[serde(rename = "mapbox")]
    Mapbox,

    // 'terrarium' string
    #[serde(rename = "terrarium")]
    Terrarium,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UTileJSON {
    #[serde(flatten)]
    pub tj: TileJSON,

    pub minzoom: u8,
    pub maxzoom: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terrain: Option<Terrain>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<String>,
}

/// Set any missing default values per tile-json specification
impl UTileJSON {
    pub fn set_missing_defaults(&mut self) {
        self.tj.set_missing_defaults();

        if self.tj.tilejson.is_empty() {
            self.tj.tilejson = "3.0.0".to_string();
        }
        if self.tj.tiles.is_empty() {
            self.tj.tiles = vec![];
        }
        if self.tj.vector_layers.is_none() {
            self.tj.vector_layers = None;
        }
    }
}
impl Deref for UTileJSON {
    type Target = TileJSON;

    fn deref(&self) -> &Self::Target {
        &self.tj
    }
}

/// # Panics
///
/// Panics from `serde_json::to_string_pretty` or `serde_json::to_string`
#[must_use]
pub fn tilejson_stringify(tj: &TileJSON, fmt: Option<bool>) -> String {
    match fmt {
        Some(false) => serde_json::to_string(&tj).expect("tilejson_stringify failed"),
        _ => serde_json::to_string_pretty(&tj)
            .expect("tilejson_stringify failed (pretty-print)"),
    }
}

pub fn tilejson_parse(s: &str) -> Result<TileJSON, serde_json::Error> {
    serde_json::from_str(s)
}

fn to_val<V, E: Display>(val: Result<V, E>) -> Option<V> {
    val.ok()
}

/// Convert metadata rows to a `TileJSON` object
/// (ripped from martin-mbtiles thank y'all very much)
pub fn metadata2tilejson(metadata: Vec<MbtMetadataRow>) -> UtilesResult<TileJSON> {
    let mut tj = tilejson! {tiles : vec![]};
    let mut json: Option<JSONValue> = None;

    for row in metadata {
        let name = row.name;
        let value = row.value;
        match name.as_ref() {
            "name" => tj.name = Some(value),
            "version" => tj.version = Some(value),
            "bounds" => tj.bounds = Bounds::from_str(value.as_str()).ok(),
            "center" => tj.center = Center::from_str(value.as_str()).ok(),
            "minzoom" => tj.minzoom = value.parse().ok(),
            "maxzoom" => tj.maxzoom = value.parse().ok(),
            "description" => tj.description = Some(value),
            "attribution" => tj.attribution = Some(value),
            "legend" => tj.legend = Some(value),
            "template" => tj.template = Some(value),
            "json" => json = to_val(serde_json::from_str(&value)),
            _ => {
                let parsed = serde_json::from_str(&value);
                if let Ok(parsed) = parsed {
                    tj.other.insert(name, parsed);
                } else {
                    tj.other.insert(name, Value::String(value));
                }
            }
        }
    }

    if let Some(JSONValue::Object(obj)) = &mut json {
        if let Some(value) = obj.remove("vector_layers") {
            if let Ok(v) = serde_json::from_value(value) {
                tj.vector_layers = Some(v);
            }
        }
        if let Some(value) = obj.remove("tilestats") {
            if let Ok(v) = serde_json::from_value::<TileStats>(value) {
                let key = "tilestats".to_string();
                let val = serde_json::to_value(v);
                // try to insert the tilestats into the other field
                if let Ok(val) = val {
                    tj.other.insert(key, val);
                }
            }
        }
        // insert the rest
        {
            let keys = obj.keys().cloned().collect::<Vec<String>>();
            for key in keys {
                if let Some(value) = obj.remove(&key) {
                    tj.other.insert(key, value);
                }
            }
        }
    }
    Ok(tj)
}

// crate::tilejson maro:
// ```
// #[macro_export]
// macro_rules! tilejson {
//     ( tilejson: $ver:expr, tiles: $sources:expr $(, $tag:tt : $val:expr)* $(,)? ) => {
//         $crate::TileJSON {
//             $( $tag: Some($val), )*
//             ..$crate::TileJSON {
//                 tilejson: $ver,
//                 tiles: $sources,
//                 vector_layers: None,
//                 attribution: None,
//                 bounds: None,
//                 center: None,
//                 data: None,
//                 description: None,
//                 fillzoom: None,
//                 grids: None,
//                 legend: None,
//                 maxzoom: None,
//                 minzoom: None,
//                 name: None,
//                 scheme: None,
//                 template: None,
//                 version: None,
//                 other: Default::default(),
//             }
//         }
//     };
//     ( tiles: $sources:expr $(, $tag:tt : $val:expr)* $(,)? ) => {
//         $crate::tilejson! {
//             tilejson: "3.0.0".to_string(),
//             tiles: $sources,
//             $( $tag: $val , )* }
//     };
//     ( $tile_source:expr $(, $tag:tt : $val:expr)* $(,)? ) => {
//         $crate::tilejson! {
//             tiles: vec! [ $tile_source ],
//             $( $tag: $val , )* }
//     };
// }
// ```
#[macro_export]
macro_rules! utilejson {
    ( tiles: $tile_source:expr, minzoom: $minzoom:expr, maxzoom: $maxzoom:expr $(, $tag:tt : $val:expr)* $(,)?) => {
        $crate::utilejson::UTileJSON {
            tj: tilejson::tilejson! {
                tiles: $tile_source,
                $( $tag : $val, )*
            },
            minzoom: $minzoom,
            maxzoom: $maxzoom,
            terrain: None,
            id: None,
            generator: None,
        }
    };
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used)]

    use std::collections::BTreeMap;

    use tilejson::TileJSON;

    use crate::utilejson::{Terrain, UTileJSON};

    #[test]
    pub(super) fn test_utilejson_stringify() {
        // Example TileJSON instance
        let tile_json = TileJSON {
            tilejson: "3.0.0".to_string(),
            name: Some("Example TileJSON".to_string()),
            scheme: None,
            template: None,
            version: None,
            tiles: vec!["https://example.com/{z}/{x}/{y}.png".to_string()],
            vector_layers: None,
            attribution: None,
            bounds: None,
            center: None,
            data: None,
            description: None,
            fillzoom: None,
            grids: None,
            legend: None,
            maxzoom: None,
            minzoom: None,
            other: BTreeMap::default(),
        };

        // Create an instance of your wrapped struct
        let utile_json = UTileJSON {
            tj: tile_json,
            terrain: Some(Terrain::Mapbox),
            minzoom: 0,
            maxzoom: 22,
            id: None,
            generator: None,
        };
        let string = serde_json::to_string(&utile_json).unwrap();

        let expected = "{\"tilejson\":\"3.0.0\",\"tiles\":[\"https://example.com/{z}/{x}/{y}.png\"],\"name\":\"Example TileJSON\",\"minzoom\":0,\"maxzoom\":22,\"terrain\":\"mapbox\"}";
        assert_eq!(string, expected);
        let parsed = serde_json::from_str::<UTileJSON>(&string).unwrap();
        assert_eq!(parsed.terrain, Some(Terrain::Mapbox));
        // let tj_str = super::tilejson_stringify(&tj, None);
        // assert_eq!(tj_str, "{\"tiles\":[]}");
    }

    #[test]
    pub(super) fn test_utilejson_macros() {
        let utj = utilejson! {
            tiles: vec!["https://example.com/{z}/{x}/{y}.png".to_string()],
            minzoom: 0,
            maxzoom: 30,
            name: "Example TileJSON".to_string(),
            scheme: "xyz".to_string(),
        };

        assert_eq!(
            utj.tj.tiles,
            vec!["https://example.com/{z}/{x}/{y}.png".to_string()]
        );
        assert_eq!(utj.terrain, None);
        assert_eq!(utj.minzoom, 0);
        assert_eq!(utj.maxzoom, 30);
        assert_eq!(utj.tj.name, Some("Example TileJSON".to_string()));
        assert_eq!(utj.tj.scheme, Some("xyz".to_string()));
    }
}
