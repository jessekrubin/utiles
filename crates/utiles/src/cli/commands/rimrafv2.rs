use std::cell::Cell;
use std::path::Path;

use futures::stream::{self, StreamExt};
use tokio::fs;
use tracing::error;
use walkdir::{DirEntry, WalkDir};

use crate::args::RimrafArgs;
// use std::path::Path;
// use tokio::fs;
use tokio::sync::mpsc;
// use tracing::error;
// use walkdir::{DirEntry, WalkDir};

// use crate::args::RimrafArgs;

async fn is_empty_dir(dirpath: &Path) -> bool {
    let mut entries = fs::read_dir(dirpath).await.unwrap();
    match entries.next_entry().await {
        Ok(None) => true,
        Ok(Some(_)) => false,
        Err(e) => {
            error!("Error reading dir: {:?}", e);
            false
        }
    }
}

pub enum DirEntryType {
    File,
    Dir,
}

#[derive(Debug)]
struct DirEntriesStats {
    pub nfiles: u64,
    pub ndirs: u64,
    pub nbytes: u64,
}

pub async fn rimraf_main_v2(args: RimrafArgs) {
    println!("rimraf_main: args: {args:?}");

    let dirpath = Path::new(&args.dirpath);
    if !dirpath.exists() {
        error!("dirpath does not exist: {:?}", dirpath);
        return;
    }

    // .filter(|e| e.file_type().is_file());

    let (tx, mut rx) = mpsc::channel::<(u64, DirEntryType)>(32);

    // Spawn a task to accumulate file sizes
    let accumulator = tokio::spawn(async move {
        let mut nfiles = 0u64;
        let mut ndirs = 0u64;
        let mut total_size = 0u64;

        while let Some((filesize, detype)) = rx.recv().await {
            match detype {
                DirEntryType::File => {
                    total_size += filesize;
                    nfiles += 1;
                }
                DirEntryType::Dir => {
                    ndirs += 1;
                }
            }
            println!(
                "nfiles: {}, ndirs: {}, total_size: {}",
                nfiles, ndirs, total_size
            );
        }
        Ok::<DirEntriesStats, ()>(DirEntriesStats {
            nfiles,
            ndirs,
            nbytes: total_size,
        })
    });
    let rm_file_dir = |file: DirEntry, mut tx: mpsc::Sender<(u64, DirEntryType)>| async move {
        // if it is a file:
        if file.file_type().is_file() {
            let filesize = fs::metadata(file.path()).await.unwrap().len();
            fs::remove_file(file.path()).await.unwrap();
            tx.send((filesize, DirEntryType::File)).await.unwrap();
            // tx.send(filesize).await.unwrap();
        } else if file.file_type().is_dir() {
            if is_empty_dir(file.path()).await {
                fs::remove_dir(file.path()).await.unwrap();
            }
            tx.send((0u64, DirEntryType::Dir)).await.unwrap();
        }
        // let filesize = fs::metadata(file.path()).await.unwrap().len();
        // fs::remove_file(file.path()).await.unwrap();
        // tx.send(filesize).await.unwrap();
    };

    let dirs_iter = WalkDir::new(args.clone().dirpath.clone())
        .contents_first(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_dir());
    let dirs_stream = stream::iter(dirs_iter);
    dirs_stream
        .for_each_concurrent(10, |file| {
            let tx = tx.clone();
            async move {
                rm_file_dir(file, tx).await;
            }
        })
        .await;

    // Drop the sender to close the channel and let the accumulator finish
    drop(tx);

    // go through the thing again and remove empty dirs
    // let files_iter = WalkDir::new(args.clone().dirpath.clone())

    // Wait for the accumulator to finish processing
    match accumulator.await.unwrap() {
        Ok(stats) => {
            println!("stats: {:?}", stats);
        }
        Err(e) => {
            error!("Error accumulating stats: {:?}", e);
        }
    }
}
