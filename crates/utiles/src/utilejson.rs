use tilejson::TileJSON;

/// # Panics
///
/// Panics from `serde_json::to_string_pretty` or `serde_json::to_string`
#[must_use]
pub fn tilejson_stringify(tj: &TileJSON, fmt: Option<bool>) -> String {
    match fmt {
        Some(false) => serde_json::to_string(&tj).unwrap(),
        _ => serde_json::to_string_pretty(&tj).unwrap(),
    }
}

pub fn tilejson_parse(s: &str) -> Result<TileJSON, serde_json::Error> {
    serde_json::from_str(s)
}