use crate::cli::stdinterator::StdInterator;
use std::io;

pub fn stdin_filtered(
    input: Option<String>,
) -> Box<dyn Iterator<Item = io::Result<String>>> {
    let input_lines = StdInterator::new(input);
    let filtered_lines = input_lines
        .filter(|l| !l.is_err())
        .filter(|l| !l.as_ref().unwrap().is_empty())
        .filter(|l| l.as_ref().unwrap() != "\x1e");
    Box::new(filtered_lines)
}
