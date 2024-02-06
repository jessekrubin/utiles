use std::collections::HashMap;
use std::sync::Arc;

use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::utilesqlite::mbtiles_async_sqlite::MbtilesAsyncSqlitePool;
use axum::extract::Path;
use axum::{
    body::{Body, Bytes},
    extract::Request,
    extract::State,
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, warn};
use utiles_core::tile_type::blob2headers;
use utiles_core::{quadkey2tile, utile, Tile};

//=============================================================================

pub struct Datasets {
    pub mbtiles: HashMap<String, MbtilesAsyncSqlitePool>,
}

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
    pub fn new(host: String, port: u16, fspaths: Vec<String>) -> Self {
        Self {
            host,
            port,
            fspaths,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

async fn preflight(config: &UtilesServerConfig) -> Datasets {
    warn!("__PREFLIGHT__");
    debug!("preflight fspaths: {:?}", config.fspaths);
    let mut datasets = HashMap::new();
    for fspath in config.fspaths.iter() {
        let pool = MbtilesAsyncSqlitePool::open(fspath).await.unwrap();
        datasets.insert(pool.filename().to_string().replace(".mbtiles", ""), pool);
    }

    // print the datasets
    for k in datasets.keys() {
        info!("{}", k);
    }

    Datasets { mbtiles: datasets }
}

pub async fn utiles_serve() -> Result<(), Box<dyn std::error::Error>> {
    warn!("__UTILES_SERVE__");

    // tmp fspath(s) hard  coded
    let fspaths = vec![
        "D:\\blue-marble\\blue-marble.mbtiles".to_string(),
        "D:\\maps\\reptiles\\mbtiles\\faacb\\20230420\\sec-crop\\Seattle_SEC_20230420_c98.mbtiles".to_string(),
    ];

    let cfg = UtilesServerConfig::new("0.0.0.0".to_string(), 3333, fspaths);

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

    // Build the app/router!
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/uitiles", get(uitiles))
        .route("/datasets", get(|_: Request<Body>| async { "datasets" }))
        .route("/tiles/:dataset/tile.json", get(ds_tilejson))
        .route("/tiles/:dataset/:quadkey", get(handle_tile_quadkey))
        .route("/tiles/:dataset/:z/:x/:y", get(tile_zxy_path))
        .layer(axum::middleware::from_fn(print_request_response))
        .with_state(shared_state) // shared app/server state
        .fallback(four_o_four); // 404

    // let addr = cfg.addr();
    info!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct TileZxyPath {
    dataset: String,
    z: u8,
    x: u32,
    y: u32,
}

async fn tile_zxy_path(
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
    let mbtiles = mbtiles.unwrap();
    let tile_data = mbtiles.query_tile(t).await.unwrap();
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

async fn handle_tile_quadkey(
    State(state): State<Arc<ServerState>>,
    Path(path): Path<TileQuadkeyPath>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mbtiles = state.datasets.mbtiles.get(&path.dataset).unwrap();
    let parsed_tile = quadkey2tile(&path.quadkey).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error parsing quadkey: {}", e),
        )
    })?;
    let tile_data = mbtiles.query_tile(parsed_tile).await.unwrap();
    // info!("tile_data: {:?}", tile_data);
    match tile_data {
        Some(data) => Ok(Response::new(Body::from(data))),
        None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
    }
}

async fn ds_tilejson(
    State(state): State<Arc<ServerState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let dataset = path;
    let mbtiles = state.datasets.mbtiles.get(&dataset).unwrap();
    let tilejson = mbtiles.tilejson().await.unwrap();
    Json(tilejson)
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
    // let v =
    //     json!({"status": "TODO/WIP/NOT_IMPLEMENTED_YET"});
    Json(json!({"status": "TODO/WIP/NOT_IMPLEMENTED_YET"}))
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));
    Ok(res)
}

async fn buffer_and_print<B>(
    direction: &str,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };
    if let Ok(body) = std::str::from_utf8(&bytes) {
        info!("{direction} body = {body:?}");
    }
    Ok(bytes)
}
