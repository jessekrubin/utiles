use crate::{UtilesError, UtilesResult};
use tracing::error;
use utiles_core::UtilesCoreError;

pub fn unimplemented_cmd_main(cmd: &str) -> UtilesResult<()> {
    error!("COMMAND NOT IMPLEMENTED {}", cmd);
    Err(UtilesError::Unimplemented(cmd.to_string()))
}
