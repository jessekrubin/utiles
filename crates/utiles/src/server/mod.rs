use axum_extra::TypedHeader;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

use axum::extract::Path;
use axum::http::Method;
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{
        StatusCode,
        header::{HeaderMap, HeaderValue},
    },
    response::IntoResponse,
    routing::get,
};
use headers::Host;
use serde::Deserialize;
use serde_json::json;
use tilejson::TileJSON;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnFailure};
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;

use request_id::Radix36MakeRequestId;
use utiles_core::tile_type::{TileKind, blob2headers};
use utiles_core::{Tile, quadkey2tile, utile};

use crate::errors::UtilesResult;
use crate::internal::signal::shutdown_signal;
use crate::mbt::MbtilesAsync;
pub use crate::server::cfg::UtilesServerConfig;
use crate::server::health::Health;
use crate::server::preflight::preflight;
use crate::server::state::{MbtilesDataset, ServerState};
use crate::server::ui::uitiles;

mod cfg;
mod favicon;
mod health;
mod preflight;
pub mod radix36;
mod request_id;
mod state;
mod ui;

pub async fn utiles_serve(cfg: UtilesServerConfig) -> UtilesResult<()> {
    info!("__UTILES_SERVE__");
    let utiles_serve_config_json = serde_json::to_string_pretty(&cfg)?;
    info!("config:\n{}", utiles_serve_config_json);

    let addr = cfg.addr();
    let datasets = preflight(&cfg).await?;
    let start = std::time::Instant::now();
    let state = ServerState {
        config: cfg,
        datasets,
        start_ts: start,
    };
    // Wrap state in an Arc so that it can be shared with the app...
    // ...seems to be the idiomatic way to do this...
    let shared_state = Arc::new(state);
    let compression_layer: CompressionLayer =
        CompressionLayer::new().gzip(true).zstd(true);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_body_chunk(DefaultOnBodyChunk::new())
        .on_failure(DefaultOnFailure::new())
        .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
            let request_headers = request.headers();
            tracing::info!(
                method = ?request.method(),
                uri = ?request.uri(),
                request_headers = ?request_headers,
                "incoming request"
            );
        })
        .on_response(
            |response: &axum::http::Response<_>,
             latency: Duration,
             _span: &tracing::Span| {
                let response_headers = response.headers();
                tracing::info!(
                    status = ?response.status(),
                    latency = ?latency.as_millis(),
                    response_headers = ?response_headers,
                    "request completed"
                );
            },
        );
    let cors_layer = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);
    // Build our middleware stack
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(
            Radix36MakeRequestId::default(),
        ))
        // propagate `x-request-id` headers from request to response
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(1),
        ))
        .layer(compression_layer);

    // Build the app/router!
    let app = Router::new()
        .route("/", get(root))
        .route("/favicon.ico", get(favicon::favicon))
        .route("/health", get(health))
        .route("/cfg", get(get_cfg))
        .route("/uitiles", get(uitiles))
        .route("/datasets", get(get_datasets))
        .route("/tiles/{dataset}/tile.json", get(get_dataset_tilejson))
        .route(
            "/tiles/{dataset}/qk/{quadkey}",
            get(get_dataset_tile_quadkey),
        )
        .route("/tiles/{dataset}/{z}/{x}/{y}", get(get_dataset_tile_zxy))
        .layer(trace_layer)
        .layer(middleware)
        .layer(cors_layer)
        .with_state(shared_state) // shared app/server state
        .fallback(four_o_four); // 404

    // let addr = cfg.addr();
    info!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
// =====================================================================
// ROUTES ~ ROUTES ~ ROUTES ~ ROUTES ~ ROUTES ~ ROUTES ~ ROUTES ~ ROUTES
// =====================================================================
#[derive(Deserialize)]
struct TileZxyPath {
    dataset: String,
    z: u8,
    x: u32,
    y: u32,
}

enum GetTileResponse {
    Data(Vec<u8>),
    NotFound,
    NoContent,
}

async fn dataset_query_tile(
    dataset: &MbtilesDataset,
    tile: &Tile,
) -> anyhow::Result<GetTileResponse> {
    let tile_data = dataset.mbtiles.query_tile(tile).await?;
    match tile_data {
        Some(data) => Ok(GetTileResponse::Data(data)),
        None => {
            if dataset.tilekind == TileKind::Vector {
                Ok(GetTileResponse::NoContent)
            } else {
                Ok(GetTileResponse::NotFound)
            }
        }
    }
}

async fn get_dataset_tile_zxy(
    State(state): State<Arc<ServerState>>,
    Path(path): Path<TileZxyPath>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mbtiles = state.datasets.mbtiles.get(&path.dataset);
    let t = utile!(path.x, path.y, path.z);
    if let Some(mbt_ds) = mbtiles {
        let tile_data = dataset_query_tile(mbt_ds, &t).await;

        match tile_data {
            Ok(GetTileResponse::Data(data)) => {
                let headers = blob2headers(&data);
                let mut headers_map = HeaderMap::new();
                for (k, v) in headers {
                    if let Ok(hvalue) = HeaderValue::from_str(v) {
                        headers_map.insert(k, hvalue);
                    }
                }
                Ok((StatusCode::OK, headers_map, Body::from(data)))
            }
            Ok(GetTileResponse::NoContent) => {
                Ok((StatusCode::NO_CONTENT, HeaderMap::new(), Body::empty()))
            }
            Ok(GetTileResponse::NotFound) => Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Tile not found",
                    "dataset": path.dataset,
                    "tile": t,
                    "status": 404,
                }))
                .to_string(),
            )),
            Err(e) => Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": e.to_string(),
                    "dataset": path.dataset,
                    "tile": t,
                    "status": 404,
                }))
                .to_string(),
            )),
        }
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Dataset not found",
                "dataset": path.dataset,
                "status": 404,
            }))
            .to_string(),
        ))
    }
}

#[derive(Deserialize)]
struct TileQuadkeyPath {
    dataset: String,
    quadkey: String,
}

async fn get_dataset_tile_quadkey(
    State(state): State<Arc<ServerState>>,
    Path(path): Path<TileQuadkeyPath>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mbt_ds = state.datasets.mbtiles.get(&path.dataset);
    if let Some(mbt_ds) = mbt_ds {
        let parsed_tile = quadkey2tile(&path.quadkey).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Error parsing quadkey: {e}"),
            )
        })?;
        let tile_data = dataset_query_tile(mbt_ds, &parsed_tile).await;
        match tile_data {
            Ok(GetTileResponse::Data(data)) => {
                let headers = blob2headers(&data);
                let mut headers_map = HeaderMap::new();
                for (k, v) in headers {
                    if let Ok(hvalue) = HeaderValue::from_str(v) {
                        headers_map.insert(k, hvalue);
                    }
                }
                Ok((StatusCode::OK, headers_map, Body::from(data)))
            }
            Ok(GetTileResponse::NoContent) => {
                Ok((StatusCode::NO_CONTENT, HeaderMap::new(), Body::empty()))
            }
            Ok(GetTileResponse::NotFound) => Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Tile not found",
                    "dataset": path.dataset,
                    "tile": parsed_tile,
                    "status": 404,
                }))
                .to_string(),
            )),
            Err(e) => Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": e.to_string(),
                    "dataset": path.dataset,
                    "tile": parsed_tile,
                    "status": 404,
                }))
                .to_string(),
            )),
        }
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Dataset not found",
                "dataset": path.dataset,
                "status": 404,
            }))
            .to_string(),
        ))
    }
}

async fn get_datasets(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let r = state
        .datasets
        .mbtiles
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    Json(r)
}

async fn get_dataset_tilejson(
    TypedHeader(host): TypedHeader<Host>,
    State(state): State<Arc<ServerState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let dataset = path;

    let ds = state.datasets.mbtiles.get(&dataset);
    if let Some(ds) = ds {
        let tilejson = ds.tilejson.clone();
        let tiles_url = format!("http://{host}/tiles/{dataset}/{{z}}/{{x}}/{{y}}");
        let tilejson_with_tiles = TileJSON {
            tiles: vec![tiles_url],
            ..tilejson
        };
        if let Ok(tjval) = serde_json::to_value(&tilejson_with_tiles) {
            Json(tjval)
        } else {
            Json(json!({
                "error": "Error serializing tilejson",
                "status": 500,
            }))
        }
    } else {
        Json(json!({
            "error": "Dataset not found",
            "dataset": dataset,
            "status": 404,
        }))
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "utiles"
}

async fn four_o_four() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "four oh four...")
}

async fn health(State(state): State<Arc<ServerState>>) -> Json<Health> {
    let uptime = std::time::Instant::now()
        .duration_since(state.start_ts)
        .as_secs();
    Json(Health::new("OK".to_string(), uptime))
}

async fn get_cfg(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let cfg_val =
        serde_json::to_value(&state.config).expect("Error serializing config");
    Json(cfg_val)
}
