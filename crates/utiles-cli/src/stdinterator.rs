use std::io;
use std::io::BufRead;
use tracing::debug;

pub enum StdInteratorSource {
    Single(String),
    Multiple(Box<dyn BufRead>),
}

pub struct StdInterator {
    source: StdInteratorSource,
}

impl StdInterator {
    pub fn new(input: Option<String>) -> io::Result<Self> {
        let source = match input {
            Some(file_content) => {
                if file_content == "-" {
                    debug!("reading from stdin - got '-'");
                    let reader = Box::new(io::BufReader::new(io::stdin()));
                    StdInteratorSource::Multiple(reader)
                } else {
                    debug!("reading from args: {:?}", file_content);
                    StdInteratorSource::Single(file_content)
                }
            }
            None => {
                let reader = Box::new(io::BufReader::new(io::stdin()));
                debug!("reading from stdin - no args");
                StdInteratorSource::Multiple(reader)
            }
        };
        Ok(Self { source })
    }
}

impl Iterator for StdInterator {
    type Item = io::Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            StdInteratorSource::Single(content) => {
                if content.is_empty() {
                    None
                } else {
                    Some(Ok(std::mem::take(content)))
                }
            }
            StdInteratorSource::Multiple(reader) => {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => None, // EOF
                    Ok(_) => Some(Ok(line.trim_end().to_string())),
                    Err(e) => Some(Err(e)),
                }
            }
        }
    }
}