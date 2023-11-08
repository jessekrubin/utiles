use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use serde_json::{Value as JSONValue, Value};

use crate::mbtiles::metadata_row::MbtilesMetadataRow;
use tilejson::{tilejson, Bounds, Center, TileJSON};
use tracing::{info, warn};

fn to_val<V, E: Display>(val: Result<V, E>, title: &str) -> Option<V> {
    match val {
        Ok(v) => Some(v),
        Err(_err) => {
            // let name = &self.filename;
            warn!("Unable to parse metadata {title}");
            None
        }
    }
}

/// Convert metadata rows to a TileJSON object
/// (ripped from martin-mbtiles thank y'all very much)
pub fn metadata2tilejson(
    metadata: Vec<MbtilesMetadataRow>,
) -> Result<TileJSON, Box<dyn Error>> {
    let mut tj = tilejson! {tiles : vec![]};
    // let mut layer_type: Option<String> = None;
    let mut json: Option<JSONValue> = None;

    for row in metadata {
        let name = row.name;
        let value = row.value;
        match name.as_ref() {
            "name" => tj.name = Some(value),
            "version" => tj.version = Some(value),
            "bounds" => tj.bounds = to_val(Bounds::from_str(value.as_str()), &name),
            "center" => tj.center = to_val(Center::from_str(value.as_str()), &name),
            "minzoom" => tj.minzoom = to_val(value.parse(), &name),
            "maxzoom" => tj.maxzoom = to_val(value.parse(), &name),
            "description" => tj.description = Some(value),
            "attribution" => tj.attribution = Some(value),
            // "type" => layer_type = Some(value),
            "legend" => tj.legend = Some(value),
            "template" => tj.template = Some(value),
            "json" => json = to_val(serde_json::from_str(&value), &name),
            "format" | "generator" => {
                tj.other.insert(name, Value::String(value));
            }
            _ => {
                // let file = &filename;
                // info!("{file} has an unrecognized metadata value {name}={value}");
                info!("unrecognized metadata value {name}={value}");
                tj.other.insert(name, Value::String(value));
            }
        }
    }

    if let Some(JSONValue::Object(obj)) = &mut json {
        if let Some(value) = obj.remove("vector_layers") {
            if let Ok(v) = serde_json::from_value(value) {
                tj.vector_layers = Some(v);
            } else {
                warn!(
                    "Unable to parse metadata vector_layers value",
                    // self.filename
                );
            }
        }
    }
    Ok(tj)
}
