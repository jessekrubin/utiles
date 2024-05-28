use crate::cli::stdinterator::StdInterator;
use std::io;

pub fn stdin_filtered(
    input: Option<String>,
) -> Box<dyn Iterator<Item = io::Result<String>>> {
    let input_lines = StdInterator::new(input);
    let filtered_lines = input_lines
        .filter(|l| !l.is_err())
        .filter(|l| {
            let r = l.as_ref();
            match r {
                Ok(s) => !s.is_empty(),
                Err(_) => false,
            }
        })
        .filter(|l| {
            let r = l.as_ref();
            match r {
                Ok(s) => s != "\x1e",
                Err(_) => false,
            }
        });
    Box::new(filtered_lines)
}
