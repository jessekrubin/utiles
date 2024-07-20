use tracing::debug;

use crate::cli::args::TilejsonArgs;
use crate::errors::UtilesResult;
use crate::utilejson::tilejson_stringify;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};

pub async fn tilejson_main(args: &TilejsonArgs) -> UtilesResult<()> {
    debug!("tilejson: {}", args.common.filepath);
    let mbt = MbtilesAsyncSqliteClient::open_readonly(&args.common.filepath).await?;
    let mut tj = mbt.tilejson().await?;
    if !args.tilestats {
        tj.other.remove("tilestats");
    }
    let s = tilejson_stringify(&tj, Option::from(!args.common.min));
    println!("{s}");
    Ok(())
}
