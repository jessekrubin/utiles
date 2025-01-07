use axum::Json;
use serde_json::json;

/// UI-tiles (ui) wip
pub(crate) async fn uitiles() -> Json<serde_json::Value> {
    Json(json!({"status": "TODO/WIP/NOT-IMPLEMENTED-YET"}))
}
