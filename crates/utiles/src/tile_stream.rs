use tokio_stream::wrappers::ReceiverStream;
use utiles_core::Tile;

pub(crate) type TileReceiverStream = ReceiverStream<(Tile, Vec<u8>)>;
