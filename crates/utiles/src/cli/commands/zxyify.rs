use crate::cli::args::ZxyifyArgs;
use crate::mbt::zxyify::unzxyify;
use crate::utilesqlite::mbtiles_async_sqlite::AsyncSqlite;
use crate::utilesqlite::{MbtilesAsync, MbtilesAsyncSqliteClient};
use crate::UtilesResult;

#[tracing::instrument]
pub async fn zxyify_main(args: ZxyifyArgs) -> UtilesResult<()> {
    let mbt =
        MbtilesAsyncSqliteClient::open_existing(args.common.filepath.as_str()).await?;
    mbt.assert_mbtiles().await?;

    if args.rm {
        mbt.conn(unzxyify).await?;
    } else {
        let zxy_rows_changed = mbt.zxyify().await?;
        let json_string = serde_json::to_string_pretty(&zxy_rows_changed)?;
        println!("{json_string}");
    }
    Ok(())
}
