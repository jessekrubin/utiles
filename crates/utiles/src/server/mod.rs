use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Host, Path};
use axum::http::HeaderName;
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tilejson::TileJSON;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnFailure, DefaultOnRequest};
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, info, warn};

use request_id::Radix36MakeRequestId;
use utiles_core::tile_type::blob2headers;
use utiles_core::{quadkey2tile, utile, Tile};

use crate::errors::UtilesResult;
use crate::globster::find_filepaths;
use crate::mbt::MbtilesAsync;
use crate::mbt::MbtilesClientAsync;
pub use crate::server::cfg::UtilesServerConfig;
use crate::server::health::Health;
use crate::server::state::{Datasets, MbtilesDataset, ServerState};
use crate::server::ui::uitiles;
use crate::signal::shutdown_signal;

mod cfg;
mod health;
pub mod radix36;
mod request_id;
mod state;
mod ui;

async fn preflight(config: &UtilesServerConfig) -> UtilesResult<Datasets> {
    warn!("__PREFLIGHT__");
    debug!("preflight fspaths: {:?}", config.fspaths);

    let filepaths = find_filepaths(&config.fspaths)?;
    debug!("filepaths: {:?}", filepaths);

    let mut datasets = BTreeMap::new();
    // let mut tilejsons = HashMap::new();
    for fspath in &filepaths {
        let pool = MbtilesClientAsync::open_readonly(fspath).await?;
        debug!("sanity check: {:?}", pool.filepath());
        let is_valid = pool.is_mbtiles().await;
        if is_valid.is_ok() {
            let tilejson = pool.tilejson_ext().await?;
            let filename = pool.filename().to_string().replace(".mbtiles", "");
            let mbt_ds = MbtilesDataset {
                mbtiles: pool,
                tilejson,
            };
            datasets.insert(filename, mbt_ds);
        } else {
            warn!("Skipping non-mbtiles file: {:?}", fspath);
            continue;
        }
    }

    // print the datasets
    for (k, ds) in &datasets {
        info!("{}: {}", k, ds.mbtiles.filepath());
    }

    info!("__PREFLIGHT_DONE__");

    Ok(Datasets { mbtiles: datasets })
}

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
    let x_request_id = HeaderName::from_static("x-request-id");
    let comression_layer: CompressionLayer = CompressionLayer::new()
        // .br(true)
        // .deflate(true)
        .gzip(true)
        .zstd(true);
    // .compress_when(|_, _, _, _| true);
    // Build our middleware stack
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            Radix36MakeRequestId::default(),
        ))
        // tracing/logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_body_chunk(DefaultOnBodyChunk::new())
                .on_failure(DefaultOnFailure::new())
                .on_request(DefaultOnRequest::new())
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        // propagate `x-request-id` headers from request to response
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(comression_layer);

    // Build the app/router!
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/uitiles", get(uitiles))
        .route("/datasets", get(get_datasets))
        .route("/tiles/:dataset/tile.json", get(get_dataset_tilejson))
        .route("/tiles/:dataset/:quadkey", get(get_dataset_tile_quadkey))
        .route("/tiles/:dataset/:z/:x/:y", get(get_dataset_tile_zxy))
        .layer(middleware)
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
// =============
// REQUEST ID
// =============

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

async fn get_dataset_tile_zxy(
    State(state): State<Arc<ServerState>>,
    Path(path): Path<TileZxyPath>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mbtiles = state.datasets.mbtiles.get(&path.dataset);
    if mbtiles.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Dataset not found",
                "dataset": path.dataset,
                "status": 404,
            }))
            .to_string(),
        ));
    }
    let t = utile!(path.x, path.y, path.z);
    match mbtiles {
        Some(mbt_ds) => {
            let tile_data = mbt_ds.mbtiles.query_tile(&t).await;
            match tile_data {
                Ok(data) => match data {
                    Some(data) => {
                        let headers = blob2headers(&data);
                        let mut headers_map = HeaderMap::new();

                        for (k, v) in headers {
                            if let Ok(hvalue) = HeaderValue::from_str(v) {
                                headers_map.insert(k, hvalue);
                            }
                        }
                        Ok((StatusCode::OK, headers_map, Body::from(data)))
                    }
                    None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
                },
                Err(e) => {
                    warn!("Error querying tile: {:?}", e);
                    Err((StatusCode::NOT_FOUND, "Tile not found".to_string()))
                }
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Dataset not found",
                "dataset": path.dataset,
                "status": 404,
            }))
            .to_string(),
        )),
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
    if mbt_ds.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Dataset not found",
                "dataset": path.dataset,
                "status": 404,
            }))
            .to_string(),
        ));
    }
    let parsed_tile = quadkey2tile(&path.quadkey).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error parsing quadkey: {e}"),
        )
    })?;
    if let Some(mbt_ds) = mbt_ds {
        let tile_data = mbt_ds.mbtiles.query_tile(&parsed_tile).await;
        match tile_data {
            Ok(data) => match data {
                Some(data) => {
                    let headers = blob2headers(&data);
                    let mut headers_map = HeaderMap::new();
                    for (k, v) in headers {
                        if let Ok(hvalue) = HeaderValue::from_str(v) {
                            headers_map.insert(k, hvalue);
                        }
                    }
                    Ok((StatusCode::OK, headers_map, Body::from(data)))
                }
                None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
            },
            Err(e) => {
                warn!("Error querying tile: {:?}", e);
                Err((StatusCode::NOT_FOUND, "Tile not found".to_string()))
            }
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
    Host(hostname): axum::extract::Host,
    State(state): State<Arc<ServerState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let dataset = path;
    let ds = state.datasets.mbtiles.get(&dataset);
    if let Some(ds) = ds {
        let tilejson = ds.tilejson.clone();
        let tiles_url = format!("http://{hostname}/tiles/{dataset}/{{z}}/{{x}}/{{y}}");
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
