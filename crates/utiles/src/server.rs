use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::Request, extract::State,
    http::StatusCode,
    Json,
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
    routing::get,
};
// use mbtiles::{
//     MbtilesPool
// };
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

//=============================================================================

pub struct Datasets {
    pub mbtiles: HashMap<String, String>,
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

    // pools
    // let uno_pool = MbtilesPool::new(uno).await.unwrap();

    // let metadata1 = uno_pool.get_metadata().await.unwrap();
    // debug!("metadata1: {:?}", metadata1);
    // let dos_pool = MbtilesPool::new(dos).await.unwrap();
    // let metadata2 = dos_pool.get_metadata().await.unwrap();

    // debug!("metadata2: {:?}", metadata2);
    let mut datasets = HashMap::new();
    datasets.insert("uno".to_string(),
                    uno.to_string());
    datasets.insert("dos".to_string(), dos.to_string());

    let datasets = Datasets {
        mbtiles: datasets
    };

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

    let shared_state = Arc::new(
        state
    );

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/health", get(health))
        .route("/datasets", get(|_: Request<Body>| async { "datasets" }))
        // .layer(middleware::from_fn(print_request_response))
        .with_state(shared_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
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

async fn health(
    State(state): State<Arc<ServerState>>,
) -> Json<Health> {
    let uptime = std::time::Instant::now().duration_since(state.start_ts).as_secs();
    let health = Health {
        status: "OK".to_string(),
        uptime,
    };
    Json(health)
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

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
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
