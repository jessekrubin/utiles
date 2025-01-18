use futures::stream::{self, StreamExt};
use jiff::SignedDuration;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::{
    fs,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::{error, info, trace};
use walkdir::{DirEntry, WalkDir};

use crate::cli::args::RimrafArgs;
use crate::errors::UtilesResult;
use crate::UtilesError;

/// Represents different ways we might update stats.
#[derive(Debug)]
enum StatsEvent {
    /// A file was removed. The `u64` is the file size in bytes.
    FileRemoved(u64),
    /// A directory was removed.
    DirRemoved,
}

#[derive(Debug, Clone, Copy)]
struct RimrafCfg {
    pub dryrun: bool,
    pub size: bool,
}

/// A thread-safe stats struct using atomics (optional but handy).
#[derive(Debug, Default)]
pub struct RimrafStats {
    pub nfiles: u64,
    pub ndirs: u64,
    pub nbytes: u64,
}

#[derive(Debug, Default)]
pub struct FinalRimrafStats {
    stats: RimrafStats,
    elapsed: Duration,
}

impl FinalRimrafStats {
    pub fn log(&self) {
        let nfiles = self.stats.nfiles;
        let ndirs = self.stats.ndirs;
        let nbytes = self.stats.nbytes;
        let signed_duration = SignedDuration::try_from(self.elapsed);
        let elapsed_str = signed_duration
            .map(|sd| {
                let s = format!("{:#}", sd);
                s
            })
            .unwrap_or_else(|_| "unknown".to_string());
        info!(
            "NUKED: nfiles={nfiles}, ndirs={ndirs}, nbytes={nbytes} in {elapsed_str}"
        );
    }

    pub fn json_str(&self) -> String {
        let nfiles = self.stats.nfiles;
        let ndirs = self.stats.ndirs;
        let nbytes = self.stats.nbytes;
        let signed_duration = SignedDuration::try_from(self.elapsed);
        let elapsed_str = signed_duration
            .map(|sd| {
                let s = format!("{:#}", sd);
                s
            })
            .unwrap_or_else(|_| "unknown".to_string());
        let json_str = format!(
            r#"{{"nfiles":{nfiles},"ndirs":{ndirs},"nbytes":{nbytes},"elapsed":"{elapsed_str}"}}"#,
            nfiles = nfiles,
            ndirs = ndirs,
            nbytes = nbytes,
            elapsed_str = elapsed_str,
        );
        json_str
    }
}

/// A separate task that collects stats events and updates `RimrafStats`.
async fn stats_collector(mut rx: UnboundedReceiver<StatsEvent>) -> (FinalRimrafStats) {
    let mut stats = RimrafStats::default();
    let start = std::time::Instant::now();
    while let Some(event) = rx.recv().await {
        match event {
            StatsEvent::FileRemoved(bytes) => {
                stats.nfiles += 1;
                stats.nbytes += bytes;
            }
            StatsEvent::DirRemoved => {
                stats.ndirs += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    FinalRimrafStats { stats, elapsed }
}

async fn remove_all_files(
    dirpath: &Path,
    cfg: RimrafCfg,
    tx: UnboundedSender<StatsEvent>,
) -> UtilesResult<()> {
    let file_entries = WalkDir::new(dirpath)
        .into_iter()
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().is_file());

    let file_stream = stream::iter(file_entries);
    file_stream
        .map(|entry| {
            let path = entry.path().to_owned();
            let tx = tx.clone();
            async move {
                let fsize = {
                    if cfg.size {
                        // Just gather size
                        match fs::metadata(&path).await {
                            Ok(meta) => meta.len(),
                            Err(e) => {
                                trace!(
                                    "Failed to get metadata on file {:?}: {:?}",
                                    path,
                                    e
                                );
                                0
                            }
                        }
                    } else {
                        0
                    }
                };

                if !cfg.dryrun {
                    // Remove file
                    match fs::remove_file(&path).await {
                        Ok(_) => {
                            // Attempt to re-check size
                            match path.metadata() {
                                Ok(meta) => {
                                    let _ =
                                        tx.send(StatsEvent::FileRemoved(meta.len()));
                                }
                                Err(_) => {
                                    let _ = tx.send(StatsEvent::FileRemoved(0));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Removing file {:?} failed: {:?}", path, e);
                        }
                    }
                } else {
                    // Just gather size
                    let _ = tx.send(StatsEvent::FileRemoved(fsize));
                }
            }
        })
        .buffer_unordered(10) // concurrency for file removal
        .collect::<()>()
        .await;
    Ok(())
}

async fn remove_all_directories_in_stages(
    dirpath: &Path,
    cfg: RimrafCfg,
    tx: UnboundedSender<StatsEvent>,
) -> UtilesResult<()> {
    if cfg.dryrun {
        return Ok(());
    }

    let mut depth_map: BTreeMap<usize, Vec<PathBuf>> = BTreeMap::new();

    // build the depth map
    for entry_result in WalkDir::new(dirpath) {
        if let Ok(entry) = entry_result {
            if entry.file_type().is_dir() {
                // dirs.push(entry);
                let path = entry.path().to_owned();
                let depth = path.components().count(); // number of components
                depth_map.entry(depth).or_default().push(path);
            }
        }
    }

    let mut depths: Vec<usize> = depth_map.keys().copied().collect();
    depths.sort_unstable_by(|a, b| b.cmp(a));
    for depth in depths {
        let paths_at_depth = depth_map.remove(&depth).unwrap_or_default();
        let dir_stream_at_depth = stream::iter(paths_at_depth);
        // Then we remove them concurrently:
        dir_stream_at_depth
            .map(|path| {
                let tx = tx.clone();
                async move {
                    match fs::remove_dir(&path).await {
                        Ok(_) => {
                            // Send DirRemoved event
                            let _ = tx.send(StatsEvent::DirRemoved);
                        }
                        Err(e) => {
                            error!("Removing directory {:?} failed: {:?}", path, e);
                        }
                    }
                }
            })
            .buffer_unordered(10)
            .collect::<()>()
            .await;
    }
    Ok(())
}

/// NUKE A DIRECTORY!
///
/// Does this:
/// 1) Remove all files
/// 2) Remove all directories
///     2a) gathers all dirs and then sorts by depth into map
///     2b) removes dirs in descending order of depth
pub async fn rimraf_main(args: RimrafArgs) -> UtilesResult<()> {
    trace!("rimraf_main: args = {:?}", args);
    let dirpath = Path::new(&args.dirpath);
    if !dirpath.exists() {
        error!("Path does not exist: {:?}", dirpath);
        return Err(UtilesError::Error(format!(
            "dirpath does not exist: {dirpath:?}"
        )));
    }

    // channel 4 collector
    let (tx, rx) = mpsc::unbounded_channel();
    let stats_handle: JoinHandle<_> =
        tokio::spawn(async move { stats_collector(rx).await });

    let cfg = RimrafCfg {
        dryrun: args.dryrun,
        size: args.size,
    };
    // remove all the files
    remove_all_files(dirpath, cfg, tx.clone()).await?;

    // remove all the dirs
    remove_all_directories_in_stages(dirpath, cfg, tx.clone()).await?;

    // boom done
    drop(tx);

    // get final stats...
    let final_stats: FinalRimrafStats = stats_handle
        .await
        .map_err(|e| UtilesError::Error(format!("Stats collector task failed: {e}")))?;

    // let (nfiles, ndirs, nbytes) = final_stats.snapshot();
    if args.verbose {
        final_stats.log();
    }
    Ok(())
}
