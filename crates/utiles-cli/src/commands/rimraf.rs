use std::cell::Cell;
use std::path::Path;

use futures::stream::{self, StreamExt};
use tokio::fs;
use tracing::error;
use walkdir::{DirEntry, WalkDir};

use crate::args::RimrafArgs;

// pub enum DirEntryType {
//     File,
//     Dir,
// }

// #[derive(Debug)]
// struct DirEntriesStats {
//     pub nfiles: u64,
//     pub ndirs: u64,
//     pub nbytes: u64,
// }

// pub async fn rimraf_main_v2(args: RimrafArgs) {
//     println!("rimraf_main: args: {args:?}");
//
//     let dirpath = Path::new(&args.dirpath);
//     if !dirpath.exists() {
//         error!("dirpath does not exist: {:?}", dirpath);
//         return;
//     }
//
//     // .filter(|e| e.file_type().is_file());
//
//     let (tx, mut rx) = mpsc::channel::<(u64, DirEntryType)>(32);
//
//     // Spawn a task to accumulate file sizes
//     let accumulator = tokio::spawn(async move {
//         let mut nfiles = 0u64;
//         let mut ndirs = 0u64;
//         let mut total_size = 0u64;
//
//         while let Some((filesize, detype)) = rx.recv().await {
//             match detype {
//                 DirEntryType::File => {
//                     total_size += filesize;
//                     nfiles += 1;
//                 }
//                 DirEntryType::Dir => {
//                     ndirs += 1;
//                 }
//             }
//             println!(
//                 "nfiles: {}, ndirs: {}, total_size: {}",
//                 nfiles, ndirs, total_size
//             );
//         }
//         Ok::<DirEntriesStats, ()>(DirEntriesStats {
//             nfiles,
//             ndirs,
//             nbytes: total_size,
//         })
//     });
//     let rm_file_dir = |file: DirEntry, mut tx: mpsc::Sender<(u64, DirEntryType)>| async move {
//         // if it is a file:
//         if file.file_type().is_file() {
//             let filesize = fs::metadata(file.path()).await.unwrap().len();
//             fs::remove_file(file.path()).await.unwrap();
//             tx.send((filesize, DirEntryType::File)).await.unwrap();
//             // tx.send(filesize).await.unwrap();
//         } else if file.file_type().is_dir() {
//             if is_empty_dir(file.path()).await {
//                 fs::remove_dir(file.path()).await.unwrap();
//             }
//             tx.send((0u64, DirEntryType::Dir)).await.unwrap();
//         }
//         // let filesize = fs::metadata(file.path()).await.unwrap().len();
//         // fs::remove_file(file.path()).await.unwrap();
//         // tx.send(filesize).await.unwrap();
//     };
//
//     let dirs_iter = WalkDir::new(args.clone().dirpath.clone())
//         .contents_first(true)
//         .into_iter()
//         .filter_map(std::result::Result::ok)
//         .filter(|e| e.file_type().is_dir());
//     let dirs_stream = stream::iter(dirs_iter);
//     dirs_stream
//         .for_each_concurrent(10, |file| {
//             let tx = tx.clone();
//             async move {
//                 rm_file_dir(file, tx).await;
//             }
//         })
//         .await;
//
//     // Drop the sender to close the channel and let the accumulator finish
//     drop(tx);
//
//     // go through the thing again and remove empty dirs
//     // let files_iter = WalkDir::new(args.clone().dirpath.clone())
//
//     // Wait for the accumulator to finish processing
//     match accumulator.await.unwrap() {
//         Ok(stats) => {
//             println!("stats: {:?}", stats);
//         }
//         Err(e) => {
//             error!("Error accumulating stats: {:?}", e);
//         }
//     }
// }
//
pub async fn rimraf_main2(args: RimrafArgs) {
    println!("rimraf_main: args: {args:?}");
    // check that dirpath exists
    let dirpath = Path::new(&args.dirpath);
    if !dirpath.exists() {
        error!("dirpath does not exist: {:?}", dirpath);
        return;
    }
    let files_iter = WalkDir::new(args.clone().dirpath.clone())
        .contents_first(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file());
    let rmthing = |file: DirEntry| async move {
        let filesize = fs::metadata(file.path()).await.unwrap().len();
        // remove the file
        fs::remove_file(file.path()).await.unwrap();
        return filesize;
    };

    let s = stream::iter(files_iter);

    s.for_each_concurrent(10, |file| async {
        let filesize = rmthing(file).await;
    })
        .await;
    // remove the dirpath

    fs::remove_dir_all(&args.dirpath).await.unwrap();
}
// iter files...

#[derive(Debug)]
pub struct RimrafStats {
    pub nfiles: Cell<u64>,
    pub ndirs: Cell<u64>,
    pub nbytes: Cell<u64>,
}

impl RimrafStats {
    pub fn new() -> Self {
        Self {
            nfiles: Cell::new(0),
            ndirs: Cell::new(0),
            nbytes: Cell::new(0),
        }
    }
    pub fn inc_nfiles(&self) {
        self.nfiles.set(self.nfiles.get() + 1);
    }

    // pub fn inc_ndirs(&self) {
    //     self.ndirs.set(self.ndirs.get() + 1);
    // }

    pub fn inc_nbytes(&self, nbytes: u64) {
        self.nbytes.set(self.nbytes.get() + nbytes);
    }
}

#[derive(Debug)]
pub struct Rimrafer {
    pub cfg: RimrafArgs,
    pub stats: RimrafStats,
}

impl Rimrafer {
    pub fn new(args: RimrafArgs) -> Self {
        Self {
            cfg: args,
            stats: RimrafStats::new(),
        }
    }

    pub async fn rm_file_stats(&self, file: DirEntry) {
        let path = file.path();
        let nbytes = fs::metadata(path).await.unwrap().len();
        self.stats.inc_nfiles();
        self.stats.inc_nbytes(nbytes);
    }

    pub async fn rm_file(&self, file: DirEntry) {
        if self.cfg.size {
            self.rm_file_stats(file).await;
            self.print_stats_1000();
            return;
        }
        let path = file.path();
        fs::remove_file(path).await.unwrap();
        self.stats.inc_nfiles();
        self.print_stats_1000();
    }

    // pub async fn rm_dir(&self, dir: DirEntry) {
    //     let path = dir.path();
    //     fs::remove_dir_all(path).await.unwrap();
    //     self.stats.inc_ndirs();
    // }

    pub fn stats_str(&self) -> String {
        format!(
            "nfiles: {}, ndirs: {}, nbytes: {}",
            self.stats.nfiles.get(),
            self.stats.ndirs.get(),
            self.stats.nbytes.get()
        )
    }

    pub fn print_stats(&self) {
        println!("stats: {:?}", self.stats_str());
    }

    pub fn print_stats_1000(&self) {
        if self.stats.nfiles.get() % 1000 == 0 {
            self.print_stats();
        }
    }
}

pub async fn rimraf_main(args: RimrafArgs) {
    println!("rimraf_main: args: {args:?}");
    // check that dirpath exists
    let dirpath = Path::new(&args.dirpath);
    if !dirpath.exists() {
        error!("dirpath does not exist: {:?}", dirpath);
        return;
    }

    let files_iter = WalkDir::new(args.clone().dirpath.clone())
        .contents_first(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file());

    let rmrfer = Rimrafer::new(args);

    let s = stream::iter(files_iter);
    s.for_each_concurrent(10, |file| async {
        rmrfer.rm_file(file).await;
    })
        .await;
    fs::remove_dir_all(&rmrfer.cfg.dirpath).await.unwrap();
    rmrfer.print_stats();
}
