use crate::sqlite::{AsyncSqliteConn, RusqliteResult};
use crate::UtilesResult;
use rusqlite::Connection;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, warn};

/// Creates and returns an async `Receiver` of items derived from rows in the DB.
/// `T` is the custom output type.
/// `F` is a closure that maps a `rusqlite::Row` to `T`.
pub fn sqlite_query_tokio_receiver<T, F, C>(
    mbt: &C,
    // &MbtilesClientAsync,
    query_override: &str,
    row_mapper: F,
) -> UtilesResult<tokio::sync::mpsc::Receiver<T>>
where
    // The row_mapper must be callable from inside a `spawn_blocking`.
    F: Fn(&rusqlite::Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
    C: AsyncSqliteConn + Clone + 'static,
{
    // create a channel for streaming out `T` items
    let (tx, rx) = tokio::sync::mpsc::channel::<T>(100);
    let query = query_override.to_string();
    // clone handle for spawned task
    let mbt_clone = mbt.clone();

    tokio::spawn(async move {
        // perform the connection + row iteration on the blocking thread (via `.conn()`).
        let result = mbt_clone
            .conn(move |conn: &Connection| -> RusqliteResult<()> {
                let mut stmt = conn.prepare(&query)?;
                // map each DB row into T via `row_mapper`.
                let rows_iter = stmt.query_map([], |row| {
                    // convert row via map fn
                    let item = row_mapper(row)?;

                    // send to the channel (blocking_send is fine in this context).
                    if let Err(e) = tx.blocking_send(item) {
                        warn!("channel send error: {:?}", e);
                    }
                    Ok(())
                })?;
                // consume all rows
                for row_result in rows_iter {
                    // handling row errors here... TODO jesse
                    if let Err(e) = row_result {
                        error!("row error: {:?}", e);
                    }
                }
                Ok(())
            })
            .await;

        if let Err(e) = result {
            error!("make_stream_rx: DB error: {:?}", e);
        }
    });
    Ok(rx)
}
pub fn sqlite_query_tokio_receiver_stream<T, F, C>(
    mbt: &C,
    query_override: &str,
    row_mapper: F,
) -> UtilesResult<ReceiverStream<T>>
where
    // The row_mapper must be callable from inside a `spawn_blocking`.
    F: Fn(&rusqlite::Row) -> RusqliteResult<T> + Send + Sync + 'static,
    T: Send + 'static,
    C: AsyncSqliteConn + Clone + 'static,
{
    let tokio_rx = sqlite_query_tokio_receiver(mbt, query_override, row_mapper)?;
    Ok(ReceiverStream::new(tokio_rx))
}
