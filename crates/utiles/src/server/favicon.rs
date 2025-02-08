use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;

// const WORLD_AMERICAS_EMOJI: &str = "🌎";
// const WORLD_EUROPE_AFRICA_EMOJI: &str = "🌎";
// const WORLD_ASIA_AUSTRALIA_EMOJI: &str = "🌎";
// const MAP_EMOJI: &str = "🗺️";
// const DINO_EMOJI: &str = "🦕";
const FAVICON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"><text y="24" font-size="24">💯</text></svg>"#;
pub(super) async fn favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml"),
    );
    (StatusCode::OK, headers, FAVICON_SVG)
}
