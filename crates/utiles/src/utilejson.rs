use std::fmt::Display;
use std::str::FromStr;

use serde_json::{Value as JSONValue, Value};
use tilejson::{tilejson, Bounds, Center, TileJSON};

use utiles_core::geostats::TileStats;

use crate::errors::UtilesResult;
use crate::mbt::MbtMetadataRow;

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
    match val {
        Ok(v) => Some(v),
        Err(_err) => None,
    }
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
            "bounds" => tj.bounds = to_val(Bounds::from_str(value.as_str())),
            "center" => tj.center = to_val(Center::from_str(value.as_str())),
            "minzoom" => tj.minzoom = to_val(value.parse()),
            "maxzoom" => tj.maxzoom = to_val(value.parse()),
            "description" => tj.description = Some(value),
            "attribution" => tj.attribution = Some(value),
            "legend" => tj.legend = Some(value),
            "template" => tj.template = Some(value),
            "json" => json = to_val(serde_json::from_str(&value)),
            _ => {
                tj.other.insert(name, Value::String(value));
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
    }
    Ok(tj)
}
