use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use axum::extract::{Host, Path};
use axum::http::HeaderName;
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{HeaderMap, HeaderValue},
        Request, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tilejson::TileJSON;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnFailure, DefaultOnRequest};
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{debug, info, warn};

use utiles_core::tile_type::blob2headers;
use utiles_core::{quadkey2tile, utile, Tile};

use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::utilesqlite::mbtiles_async_sqlite::MbtilesAsyncSqlitePool;

//=============================================================================

pub struct MbtilesDataset {
    pub mbtiles: MbtilesAsyncSqlitePool,
    pub tilejson: TileJSON,
}

pub struct Datasets {
    // pub mbtiles: HashMap<String, MbtilesAsyncSqlitePool>,
    pub mbtiles: BTreeMap<String, MbtilesDataset>,
    // pub tilejsons: HashMap<String, TileJSON>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilesServerConfig {
    pub host: String,
    pub port: u16,
    pub fspaths: Vec<String>,
}

pub struct ServerState {
    pub config: UtilesServerConfig,
    pub datasets: Datasets,
    pub start_ts: std::time::Instant,
}

impl UtilesServerConfig {
    #[must_use]
    pub fn new(host: String, port: u16, fspaths: Vec<String>) -> Self {
        Self {
            host,
            port,
            fspaths,
        }
    }

    #[must_use]
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

async fn preflight(config: &UtilesServerConfig) -> Datasets {
    warn!("__PREFLIGHT__");
    debug!("preflight fspaths: {:?}", config.fspaths);
    let mut datasets = BTreeMap::new();
    // let mut tilejsons = HashMap::new();
    for fspath in config.fspaths.iter() {
        let pool = MbtilesAsyncSqlitePool::open_readonly(fspath).await.unwrap();
        let tilejson = pool.tilejson().await.unwrap();
        let filename = pool.filename().to_string().replace(".mbtiles", "");
        let mbt_ds = MbtilesDataset {
            mbtiles: pool,
            tilejson,
        };
        // datasets.insert(pool.filename().to_string().replace(".mbtiles", ""), mbt_ds);

        datasets.insert(filename, mbt_ds);
        // datasets.insert(pool.filename().to_string().replace(".mbtiles", ""), pool);
        // tilejsons.insert(pool.filename().to_string().replace(".mbtiles", ""), tilejson);
    }

    // print the datasets
    for k in datasets.keys() {
        info!("{}", k);
    }

    Datasets { mbtiles: datasets }
}

pub async fn utiles_serve(
    cfg: UtilesServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("__UTILES_SERVE__");
    let utiles_serve_config_json = serde_json::to_string_pretty(&cfg).unwrap();
    info!("config:\n{}", utiles_serve_config_json);

    let addr = cfg.addr();
    let datasets = preflight(&cfg).await;
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
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

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
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("SIGTERM received ~ shutting down... :(");
}

// =============
// REQUEST ID
// =============

/// Convert u8 to radix36 char
/// ```
/// use utiles::server::u8_radix36_char;
/// assert_eq!(u8_radix36_char(0), '0');
/// assert_eq!(u8_radix36_char(9), '9');
/// assert_eq!(u8_radix36_char(10), 'a');
/// assert_eq!(u8_radix36_char(35), 'z');
/// ```
#[must_use]
#[inline]
pub fn u8_radix36_char(num: u8) -> char {
    if num < 10 {
        (b'0' + num) as char
    } else {
        (b'a' + num - 10) as char
    }
}

/// Radix36 for request_id
///
/// ```
/// use utiles::server::u64_radix36;
/// assert_eq!(u64_radix36(0), "0");
/// assert_eq!(u64_radix36(1234), "ya");
/// assert_eq!(u64_radix36(1109), "ut");
/// assert_eq!(u64_radix36(18446744073709551615), "3w5e11264sgsf");
/// ```
#[must_use]
pub fn u64_radix36(x: u64) -> String {
    if x < 36 {
        return u8_radix36_char(x as u8).to_string();
    }
    let mut result = ['\0'; 128];
    let mut used = 1;
    let mut x = x; // as u32;
    let mut m = (x % 36) as u8;
    x /= 36;
    result[0] = u8_radix36_char(m);
    if x > 0 {
        loop {
            m = (x % 36) as u8;
            x /= 36;
            result[used] = u8_radix36_char(m);
            used += 1;
            if x == 0 {
                break;
            }
        }
    }
    let mut s = String::new();
    for c in result[..used].iter().rev() {
        s.push(*c);
    }
    s
}

#[derive(Clone, Default)]
struct Radix36MakeRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for Radix36MakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id_u64 = self.counter.fetch_add(1, Ordering::SeqCst);
        let request_id = u64_radix36(request_id_u64).parse().unwrap();
        Some(RequestId::new(request_id))
    }
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
    let mbt_ds = mbtiles.unwrap();
    let tile_data = mbt_ds.mbtiles.query_tile(t).await.unwrap();
    match tile_data {
        Some(data) => {
            let headers = blob2headers(&data);

            let mut headers_map = HeaderMap::new();
            for (k, v) in headers {
                headers_map.insert(k, HeaderValue::from_str(v).unwrap());
            }
            Ok((StatusCode::OK, headers_map, Body::from(data)))
        }
        None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
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
            format!("Error parsing quadkey: {}", e),
        )
    })?;
    let mbt_ds = mbt_ds.unwrap();
    let tile_data = mbt_ds.mbtiles.query_tile(parsed_tile).await.unwrap();
    match tile_data {
        Some(data) => Ok(Response::new(Body::from(data))),
        None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
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
    let ds = state.datasets.mbtiles.get(&dataset).unwrap();
    let mut tilejson_with_tiles = ds.tilejson.clone();
    let tiles_url = format!("http://{}/tiles/{}/{{z}}/{{x}}/{{y}}", hostname, dataset);
    tilejson_with_tiles.tiles = vec![tiles_url];
    Json(tilejson_with_tiles)
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "utiles"
}

#[derive(Serialize, Deserialize)]
struct Health {
    status: String,
    uptime: u64,
}

async fn four_o_four() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "four oh four...")
}

async fn health(State(state): State<Arc<ServerState>>) -> Json<Health> {
    let uptime = std::time::Instant::now()
        .duration_since(state.start_ts)
        .as_secs();
    let health = Health {
        status: "OK".to_string(),
        uptime,
    };
    Json(health)
}

/// UI-tiles (ui) wip
async fn uitiles() -> Json<serde_json::Value> {
    Json(json!({"status": "TODO/WIP/NOT_IMPLEMENTED_YET"}))
}
