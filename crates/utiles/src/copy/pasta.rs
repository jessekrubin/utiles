#![allow(dead_code)]

use crate::copy::CopyConfig;
use crate::errors::UtilesResult;

#[derive(Debug)]
pub struct CopyPasta {
    pub cfg: CopyConfig,
}

impl CopyPasta {
    pub fn new(cfg: CopyConfig) -> UtilesResult<CopyPasta> {
        cfg.check()?;
        // sanity check stuff here...
        Ok(Self { cfg })
    }
}
