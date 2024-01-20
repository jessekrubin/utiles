use std::cell::Cell;

use std::path::Path;

use futures::stream::{self, StreamExt};
use tokio::fs;
use tracing::error;
use walkdir::{DirEntry, WalkDir};

use crate::cli::args::RimrafArgs;

#[allow(dead_code)]
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
    let rm_fn = |file: DirEntry| async move {
        let filesize = fs::metadata(file.path()).await.unwrap().len();
        // remove the file
        fs::remove_file(file.path()).await.unwrap();
        filesize
    };

    let s = stream::iter(files_iter);

    s.for_each_concurrent(10, |file| async {
        let _filesize = rm_fn(file).await;
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
