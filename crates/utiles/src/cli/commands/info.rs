use crate::cli::args::InfoArgs;
use crate::errors::UtilesResult;
use crate::mbt::{mbinfo, MbtilesStats};
use crate::sqlite::SqliteHeader;
use crate::UtilesError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(crate) enum InfoType {
    Mbtiles,
    Pmtiles,
    Sqlite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum Info {
    Mbtiles(MbtilesStats),
    Sqlite(SqliteHeader),
}

impl InfoType {
    pub(crate) fn from_ext(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mbtiles" | "mbt" => Some(Self::Mbtiles),
            "sqlite" | "db" => Some(Self::Sqlite),
            "pmtiles" => Some(Self::Pmtiles),
            _ => None,
        }
    }

    pub(crate) async fn info(&self, filepath: &str, stats: bool) -> UtilesResult<Info> {
        let info = match self {
            Self::Mbtiles => {
                let mbtiles_info = mbinfo(filepath, Some(stats)).await?;
                Ok(Info::Mbtiles(mbtiles_info))
            }
            Self::Pmtiles => Err(UtilesError::Unimplemented(
                "pmtiles info not implemented (yet)".to_string(),
            )),
            Self::Sqlite => Err(UtilesError::Unimplemented(
                "sqlite info not implemented (yet)".to_string(),
            )),
        }?;
        Ok(info)
    }
}

pub(crate) async fn info(filepath: &str, stats: bool) -> UtilesResult<Info> {
    let ext = filepath.split('.').next_back().unwrap_or_default();
    let info_type = InfoType::from_ext(ext);
    if let Some(info_type) = info_type {
        let info = info_type.info(filepath, stats).await?;
        Ok(info)
    } else {
        Err(UtilesError::UnknownFiletype(filepath.to_string()))
    }
}

pub(crate) async fn info_main(args: &InfoArgs) -> UtilesResult<()> {
    let info = info(&args.common.filepath, args.statistics).await?;
    let str = if args.common.min {
        serde_json::to_string(&info)
    } else {
        serde_json::to_string_pretty(&info)
    }?;
    println!("{str}");
    Ok(())
}
