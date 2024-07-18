use std::path::PathBuf;

use serde::Serialize;

use crate::cli::args::CopyArgs;
use crate::errors::UtilesCopyError;
use crate::errors::UtilesResult;
use crate::mbt::hash_types::HashType;
use crate::mbt::{MbtType, TilesFilter};
use crate::sqlite::InsertStrategy;
use utiles_core::zoom::ZoomSet;
use utiles_core::BBox;

#[derive(Debug, Clone, Serialize, Default)]
pub struct CopyConfig {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub zset: Option<ZoomSet>,
    pub zooms: Option<Vec<u8>>,
    pub bboxes: Option<Vec<BBox>>,
    pub bounds_string: Option<String>,
    pub verbose: bool,
    pub dryrun: bool,
    pub force: bool,
    pub jobs: Option<u8>,
    pub istrat: InsertStrategy,
    pub dbtype: Option<MbtType>,
    pub hash: Option<HashType>,
}

impl From<&CopyArgs> for CopyConfig {
    fn from(args: &CopyArgs) -> CopyConfig {
        let dbtype = args.dbtype.as_ref().map(|dbtype| dbtype.into());
        CopyConfig {
            src: PathBuf::from(&args.src),
            dst: PathBuf::from(&args.dst),
            zset: args.zoom_set(),
            zooms: args.zooms(),
            verbose: true,
            bboxes: args.bboxes(),
            bounds_string: args.bounds(),
            force: false,
            dryrun: false,
            jobs: args.jobs,
            istrat: InsertStrategy::from(args.conflict),
            hash: args.hash,
            dbtype,
        }
    }
}

impl CopyConfig {
    pub fn src_dbpath_str(&self) -> String {
        self.src.to_string_lossy().to_string()
    }

    pub fn mbtiles_sql_where(&self) -> UtilesResult<String> {
        let tf = TilesFilter::new(self.bboxes.clone(), self.zooms.clone());
        tf.mbtiles_sql_where(None)
    }

    pub fn check_src_dst_same(&self) -> UtilesResult<()> {
        if self.src == self.dst {
            Err(
                UtilesCopyError::SrcDstSame(self.src.to_string_lossy().to_string())
                    .into(),
            )
        } else {
            Ok(())
        }
    }

    pub fn check_src_exists(&self) -> UtilesResult<()> {
        if self.src.exists() {
            Ok(())
        } else {
            Err(UtilesCopyError::SrcNotExists(format!("src: {:?}", self.src)).into())
        }
    }

    pub fn check(&self) -> UtilesResult<()> {
        self.check_src_exists()?;
        self.check_src_dst_same()?;
        Ok(())
    }

    pub fn njobs(&self) -> u8 {
        if let Some(j) = self.jobs {
            j
        } else {
            let ncpus = num_cpus::get();
            // if less than 4 cpus then use 1 job otherwise just default to 4 to
            // not throttle errything
            if ncpus < 4 {
                1
            } else {
                4
            }
        }
    }
}
