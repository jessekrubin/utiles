use tokio_stream::wrappers::ReceiverStream;
use utiles_core::Tile;

pub type TileReceiverStream = ReceiverStream<(Tile, Vec<u8>)>;
