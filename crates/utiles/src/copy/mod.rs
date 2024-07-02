use std::path::Path;

use tracing::{debug, info};

pub use cfg::CopyConfig;
pub use pyramid::copy_mbtiles2fs;
pub use unpyramid::copy_fs2mbtiles;

use crate::errors::UtilesError;
use crate::errors::UtilesResult;

mod cfg;
mod pasta;
mod pyramid;
mod unpyramid;

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
    Mbtiles2Mbtiles,
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

pub async fn copy(cfg: &CopyConfig) -> UtilesResult<()> {
    let cfg_json_str = serde_json::to_string_pretty(cfg)?;
    debug!("copy-config: {:?}", cfg);
    info!("copy-config-json: {}", cfg_json_str);

    let pasta = pasta::CopyPasta::new(cfg.clone())?;

    // check and err if necessary
    cfg.check()?;

    // TODO: figure out what I was doing here there is some duplication
    //       of things happening...
    // make sure input file exists and is file...
    let src =
        get_tile_src(pasta.cfg.src.to_str().ok_or_else(|| {
            UtilesError::Error("src is not a valid string".to_string())
        })?)?;
    let dst =
        get_tile_dst(pasta.cfg.dst.to_str().ok_or_else(|| {
            UtilesError::Error("dst is not a valid string".to_string())
        })?)?;

    let srcdst = match (src, dst) {
        (Source::Mbtiles(_src), Destination::Fs(_dst)) => Ok(CopySrcDest::Mbtiles2Fs),
        (Source::Fs(_src), Destination::Mbtiles(_dst)) => Ok(CopySrcDest::Fs2Mbtiles),
        (Source::Mbtiles(_src), Destination::Mbtiles(_dst)) => {
            Ok(CopySrcDest::Mbtiles2Mbtiles)
        }
        _ => Err(UtilesError::Unimplemented(
            "Unimplemented src/dst combination for copy/cp".to_string(),
        )),
    }?;

    match srcdst {
        CopySrcDest::Mbtiles2Fs => copy_mbtiles2fs(&pasta.cfg).await,
        CopySrcDest::Fs2Mbtiles => copy_fs2mbtiles(&pasta.cfg).await,
        CopySrcDest::Mbtiles2Mbtiles => pasta.run().await,
    }?;

    Ok(())
}
