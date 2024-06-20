use std::path::{Path, PathBuf};
use tracing::warn;

use crate::cli::args::CopyArgs;
use crate::copy::{copy, copy_fs2mbtiles, copy_mbtiles2fs, CopyConfig};
use crate::errors::{UtilesError, UtilesResult};

#[derive(Debug)]
pub enum Source {
    Mbtiles(String),
    Fs(String),
}

#[derive(Debug)]
pub enum Destination {
    Mbtiles(String),
    Fs(String),
}

pub enum CopySrcDest {
    Mbtiles2Fs,
    Fs2Mbtiles,
}

fn get_tile_src(src: &str) -> UtilesResult<Source> {
    let src_path = Path::new(src);
    if src_path.exists() {
        if src_path.is_file() {
            Ok(Source::Mbtiles(src.to_string()))
        } else if src_path.is_dir() {
            Ok(Source::Fs(src.to_string()))
        } else {
            Err(UtilesError::Error("src is not file or dir".to_string()))
        }
    } else {
        Err(UtilesError::FileDoesNotExist(src.to_string()))
    }
}

fn get_tile_dst(dst: &str) -> UtilesResult<Destination> {
    // if it contains '.mbtiles' then it's a mbtiles file
    // else it's a directory
    if dst.contains(".mbtiles") {
        Ok(Destination::Mbtiles(dst.to_string()))
    } else {
        Ok(Destination::Fs(dst.to_string()))
    }
}

pub async fn copy_main(args: CopyArgs) -> UtilesResult<()> {
    warn!("experimental command: copy/cp");

    let copy_cfg = CopyConfig {
        src: PathBuf::from(&args.src),
        dst: PathBuf::from(&args.dst),
        zset: args.zoom_set(),
        zooms: args.zooms(),
        verbose: true,
        bboxes: args.bboxes(),
        force: false,
        dryrun: false,
        jobs: args.jobs,
    };
    copy(&copy_cfg).await?;

    // TODO: figure out what I was doing here there is some duplication
    //       of things happening...
    // make sure input file exists and is file...
    let src = get_tile_src(&args.src)?;
    let dst = get_tile_dst(&args.dst)?;

    let srcdst = match (src, dst) {
        (Source::Mbtiles(_src), Destination::Fs(_dst)) => Ok(CopySrcDest::Mbtiles2Fs),
        (Source::Fs(_src), Destination::Mbtiles(_dst)) => Ok(CopySrcDest::Fs2Mbtiles),
        _ => Err(UtilesError::Unimplemented(
            "Unimplemented src/dst combination for copy/cp".to_string(),
        )),
    }?;

    match srcdst {
        CopySrcDest::Mbtiles2Fs => copy_mbtiles2fs(&copy_cfg).await,
        CopySrcDest::Fs2Mbtiles => copy_fs2mbtiles(&copy_cfg).await,
    }
}
