use crate::globster::find_filepaths;
use crate::mbt::{MbtilesAsync, MbtilesClientAsync};
use crate::server::state::{Datasets, MbtilesDataset};
use crate::server::UtilesServerConfig;
use crate::{UtilesError, UtilesResult};
use futures::{stream, StreamExt};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

async fn check_mbtiles(fspath: &PathBuf) -> UtilesResult<MbtilesDataset> {
    let mbt = MbtilesClientAsync::open_readonly(fspath).await?;
    debug!("sanity check: {:?}", mbt.filepath());
    let is_valid = mbt.is_mbtiles().await;
    match &is_valid {
        Ok(is_mbt) => {
            if !is_mbt {
                warn!("{}: is not valid mbtiles", mbt.filepath());
                return Err(UtilesError::NotMbtilesLike(
                    fspath.to_string_lossy().to_string(),
                ));
            }
            info!("{}: is valid mbtiles", mbt.filepath());
            let tilejson = mbt.tilejson_ext().await?;
            let tilekind = mbt.query_tilekind().await?;
            Ok(MbtilesDataset {
                mbtiles: mbt,
                tilejson,
                tilekind,
            })
        }
        Err(e) => {
            warn!("{}: is not valid mbtiles: {:?}", mbt.filepath(), e);
            Err(UtilesError::NotMbtilesLike(
                fspath.to_string_lossy().to_string(),
            ))
        }
    }
}

pub(crate) async fn preflight(config: &UtilesServerConfig) -> UtilesResult<Datasets> {
    let now = std::time::Instant::now();
    info!("__PREFLIGHT__ ~ starting");
    debug!("preflight fspaths: {:?}", config.fspaths);

    let filepaths = find_filepaths(&config.fspaths)?;
    debug!("filepaths: {:?}", filepaths);

    let mut datasets = BTreeMap::new();
    let mbtiles_stream = stream::iter(filepaths)
        .map(|path| async move {
            let r = check_mbtiles(&path).await;
            match r {
                Ok(ds) => {
                    let filename =
                        ds.mbtiles.filename().to_string().replace(".mbtiles", "");
                    Ok((filename, ds))
                }
                Err(e) => {
                    warn!("{}: is not valid mbtiles: {:?}", path.to_string_lossy(), e);
                    Err(e)
                }
            }
        })
        .buffer_unordered(4);
    let mbtiles = mbtiles_stream.collect::<Vec<_>>().await;
    datasets.extend(mbtiles.into_iter().filter_map(Result::ok));
    // print the datasets
    for (k, ds) in &datasets {
        info!("{}: {}", k, ds.mbtiles.filepath());
    }
    let elapsed_duration = now.elapsed();
    match jiff::Span::try_from(elapsed_duration) {
        Ok(span) => {
            info!("__PREFLIGHT__ ~ done ({:#})", span);
        }
        Err(_) => {
            info!("__PREFLIGHT__ ~ done ({:?})", elapsed_duration);
        }
    }
    Ok(Datasets { mbtiles: datasets })
}
//
// async fn preflight_og(config: &UtilesServerConfig) -> UtilesResult<Datasets> {
//     warn!("__PREFLIGHT__");
//     debug!("preflight fspaths: {:?}", config.fspaths);
//
//     let filepaths = find_filepaths(&config.fspaths)?;
//     debug!("filepaths: {:?}", filepaths);
//
//     let mut datasets = BTreeMap::new();
//     // let mut tilejsons = HashMap::new();
//     for fspath in &filepaths {
//         let mbt = MbtilesClientAsync::open_readonly(fspath).await?;
//         debug!("sanity check: {:?}", mbt.filepath());
//         let is_valid = mbt.is_mbtiles().await;
//         match &is_valid {
//             Ok(is_mbt) => {
//                 if !is_mbt {
//                     warn!("{}: is not valid mbtiles", mbt.filepath());
//                     continue;
//                 }
//                 info!("{}: is valid mbtiles", mbt.filepath());
//                 let tilejson = mbt.tilejson_ext().await?;
//                 let filename = mbt.filename().to_string().replace(".mbtiles", "");
//                 let mbt_ds = MbtilesDataset {
//                     mbtiles: mbt,
//                     tilejson,
//                 };
//                 datasets.insert(filename, mbt_ds);
//             }
//             Err(e) => {
//                 warn!("{}: is not valid mbtiles: {:?}", mbt.filepath(), e);
//             }
//         }
//     }
//
//     // print the datasets
//     for (k, ds) in &datasets {
//         info!("{}: {}", k, ds.mbtiles.filepath());
//     }
//
//     info!("__PREFLIGHT_DONE__");
//
//     Ok(Datasets { mbtiles: datasets })
// }
