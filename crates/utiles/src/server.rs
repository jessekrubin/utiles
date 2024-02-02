use std::collections::HashMap;
use std::sync::Arc;

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
// use mbtiles::{
//     MbtilesPool
// };
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use utiles_core::{quadkey2tile, utile, Tile};
use utiles_core::tile_type::{blob2headers, tiletype};

use crate::utilesqlite::mbtiles_async::MbtilesAsync;
use crate::utilesqlite::mbtiles_async_sqlite::MbtilesAsyncSqlitePool;

//=============================================================================

pub struct Datasets {
    pub mbtiles: HashMap<String, MbtilesAsyncSqlitePool>,
}

pub struct ServerState {
    pub datasets: Datasets,
    pub start_ts: std::time::Instant,
}

async fn preflight() -> Datasets {
    warn!("__PREFLIGHT__");

    let uno = "D:\\blue-marble\\blue-marble.mbtiles";
    // Seattle_SEC_20230420_c98.mbtiles
    // D:\maps\reptiles\mbtiles\faacb\20230420\sec-crop>
    let dos = "D:\\maps\\reptiles\\mbtiles\\faacb\\20230420\\sec-crop\\Seattle_SEC_20230420_c98.mbtiles";
    let dos_pool = MbtilesAsyncSqlitePool::open(dos).await.unwrap();
    let uno_pool = MbtilesAsyncSqlitePool::open(uno).await.unwrap();

    // pools
    // let uno_pool = MbtilesPool::new(uno).await.unwrap();

    // let metadata1 = uno_pool.get_metadata().await.unwrap();
    // debug!("metadata1: {:?}", metadata1);
    // let dos_pool = MbtilesPool::new(dos).await.unwrap();
    // let metadata2 = dos_pool.get_metadata().await.unwrap();

    // debug!("metadata2: {:?}", metadata2);
    let mut datasets = HashMap::new();
    let datasets_vec = vec![uno_pool, dos_pool];

    for ds in datasets_vec {
        // let tj = ds.tilejson().await.unwrap();
        // info!("tj: {:?}", tj);
        datasets.insert(
            ds.filename().clone().to_string().replace(".mbtiles", ""),
            ds,
        );
    }

    // print the datasets
    for (k, v) in &datasets {
        info!("{}", k);
    }

    // datasets.insert("uno".to_string(), uno.to_string());
    // datasets.insert("dos".to_string(), dos.to_string());
    // datasets.insert("uno".to_string(), uno_pool);
    // datasets.insert("dos".to_string(), dos_pool);

    let datasets = Datasets { mbtiles: datasets };

    datasets
}

pub async fn utiles_serve() -> Result<(), Box<dyn std::error::Error>> {
    warn!("__UTILES_SERVE__");

    let datasets = preflight().await;
    let start = std::time::Instant::now();
    let state = ServerState {
        datasets,
        start_ts: start,
    };

    // let state = axum::AddExtensionLayer::new(datasets);

    let shared_state = Arc::new(state);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/health", get(health))
        .route("/uitiles", get(uitiles))
        .route("/datasets", get(|_: Request<Body>| async { "datasets" }))
        .route("/tiles/:dataset/tile.json", get(ds_tilejson))
        .route("/tiles/:dataset/:quadkey", get(handle_tile_quadkey))
        .route("/tiles/:dataset/:z/:x/:y", get(tile_zxy_path))
        .layer(axum::middleware::from_fn(print_request_response))
        .with_state(shared_state);
    let app = app.fallback(four_o_four);

    // run our app with hyper, listening globally on port 3000
    let host = "0.0.0.0".to_string();
    let port = 3333;
    let addr = format!("{}:{}", host, port);
    info!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
// #[derive(Debug, Deserialize)]
// pub enum YParam {
//     Y(u32),
//     YExt { y: u32, ext: String },
// }

#[derive(Debug, Deserialize)]
pub enum YParam {
    Y(u32),
    YExt(String),
}

// impl std::str::FromStr for YParam {
//     type Err = std::convert::Infallible;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.parse::<u32>() {
//             Ok(num) => Ok(YParam::Y(num)),
//             Err(_) => Ok(YParam::YExt(s.to_string())),
//         }
//     }
// }
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
            let headers = blob2headers(
                &data
            );
            let hm = headers.iter().fold(HeaderMap::new(), |mut acc, (k, v)| {
                acc.insert(k.clone(), HeaderValue::from_str(v).unwrap());
                acc
            });
            // for (k, v) in &headers {
            //
            // }

            Ok(
                (
                    StatusCode::OK,
                    hm,
                    Body::from(data)
                )
            )
        }
        None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
    }

    // info!("tile_data: {:?}", tile_data);
    // match tile_data {
    //     Some(data) => Ok(Response::new(Body::from(data))),
    //     None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
    // }
    // let tile_data = mbtiles.query_tile(t).await.unwrap();
    // // info!("tile_data: {:?}", tile_data);
    // match tile_data {
    //     Some(data) => Ok(Response::new(Body::from(data))),
    //     None => Err((StatusCode::NOT_FOUND, "Tile not found".to_string())),
    // }
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
async fn uitiles(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let v = json!({"status": "OK"});
    Json(v)
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
        B: axum::body::HttpBody<Data=Bytes>,
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
