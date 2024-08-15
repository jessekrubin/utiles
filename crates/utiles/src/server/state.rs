use std::collections::BTreeMap;

use tilejson::TileJSON;

use crate::mbt::MbtilesClientAsync;
use crate::server::cfg::UtilesServerConfig;

pub struct MbtilesDataset {
    pub mbtiles: MbtilesClientAsync,
    pub tilejson: TileJSON,
}

pub struct Datasets {
    pub mbtiles: BTreeMap<String, MbtilesDataset>,
}

pub struct ServerState {
    pub config: UtilesServerConfig,
    pub datasets: Datasets,
    pub start_ts: std::time::Instant,
}
