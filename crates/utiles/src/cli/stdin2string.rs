use crate::errors::UtilesResult;
use std::io::{self, Read};

pub fn stdin2string() -> UtilesResult<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
