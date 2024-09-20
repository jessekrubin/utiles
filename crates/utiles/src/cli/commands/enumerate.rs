use crate::cli::args::EnumerateArgs;
use crate::mbt::TilesFilter;
use crate::{TileStringFormatter, UtilesResult};
use std::io;
use std::io::{BufWriter, Write};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, error};

async fn enumerate_db(
    fspath: &str,
    tformatter: TileStringFormatter,
    tfilter: &Option<TilesFilter>,
    tx: tokio::sync::mpsc::Sender<String>,
) -> UtilesResult<()> {
    let mbt = crate::mbt::MbtilesClientAsync::open_existing(fspath).await?;
    let query: String = match tfilter {
        Some(tfilter) => {
            let where_clause = tfilter.where_clause(Some("tiles."))?;
            format!(
                "SELECT zoom_level, tile_column, tile_row FROM tiles {where_clause}"
            )
        }
        None => "SELECT zoom_level, tile_column, tile_row FROM tiles".to_string(),
    };
    let s = mbt.enumerate_rx(Some(&query))?;
    let mut tiles = ReceiverStream::new(s);
    while let Some(tile) = tiles.next().await {
        // let tile_str = format!("{} {}", fspath, tile.json_arr());
        let tile_str = tformatter.fmt_tile(&tile);
        if let Err(e) = tx.send(tile_str).await {
            return Err(crate::UtilesError::Error(format!("enumerate_db: {e:?}")));
        }
    }
    Ok(())
}

pub async fn enumerate_main(args: &EnumerateArgs) -> UtilesResult<()> {
    debug!("args: {:?}", args);
    // check that all files exist...
    for fspath in &args.fspaths {
        if !std::path::Path::new(fspath).exists() {
            return Err(crate::UtilesError::Error(format!(
                "file not found: {fspath:?}"
            )));
        }
    }
    debug!("fspaths: {:?}", args.fspaths);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    let write_task: JoinHandle<Result<(), io::Error>> =
        tokio::task::spawn_blocking(move || {
            let stdout = io::stdout();
            let lock = stdout.lock();
            let mut buf = BufWriter::with_capacity(32 * 1024, lock);
            let mut count: usize = 0;
            while let Some(tile_str) = rx.blocking_recv() {
                buf.write_all(tile_str.as_bytes())?;
                buf.write_all(b"\n")?;
                count += 1;
                if count % 1024 == 0 {
                    buf.flush()?;
                    if let Err(e) = buf.flush() {
                        error!("write_task: {:?}", e);
                        break;
                    }
                }
            }
            Ok(())
        });
    let tfilter = args.filter_args.tiles_filter_maybe();
    let fspaths = args.fspaths.clone();
    let tippecanoe = args.tippecanoe;
    let enum_task: JoinHandle<UtilesResult<()>> = tokio::task::spawn(async move {
        let tf = tfilter.clone();
        let nfiles = fspaths.len();

        for fspath in fspaths {
            let formatter = if tippecanoe {
                // tippecanoe style is `{fspath} {x} {y} {z}`
                let xyz_fmt_str = "{x} {y} {z}";
                let fmt_str = format!("{fspath} {xyz_fmt_str}");
                TileStringFormatter::new(&fmt_str)
            } else if nfiles == 1 {
                TileStringFormatter::default()
            } else {
                let xyz_fmt_str = "{json_arr}";
                let fmt_str = format!("{fspath} {xyz_fmt_str}");
                TileStringFormatter::new(&fmt_str)
            };
            enumerate_db(&fspath, formatter, &tf, tx.clone()).await?;
        }
        Ok(())
    });
    let (enum_res, write_res) = tokio::try_join!(enum_task, write_task)?;
    enum_res?;
    write_res?;
    Ok(())
}