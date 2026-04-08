use std::io::{self, Read};

use crate::errors::UtilesResult;

pub(crate) fn stdin2string() -> UtilesResult<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
