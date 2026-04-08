use std::collections::BTreeMap;

use tilejson::TileJSON;
use utiles_core::tile_type::TileKind;

use crate::mbt::MbtilesClientAsync;
use crate::server::cfg::UtilesServerConfig;

#[derive(Debug)]
pub(super) struct MbtilesDataset {
    pub mbtiles: MbtilesClientAsync,
    pub tilejson: TileJSON,
    pub tilekind: TileKind,
}

#[derive(Debug)]
pub(super) struct Datasets {
    pub mbtiles: BTreeMap<String, MbtilesDataset>,
}

#[derive(Debug)]
pub(super) struct ServerState {
    pub config: UtilesServerConfig,
    pub datasets: Datasets,
    pub start_ts: std::time::Instant,
}
