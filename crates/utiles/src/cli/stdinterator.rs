use std::collections::VecDeque;
use std::io;
use std::io::BufRead;
use std::str::FromStr;
use tracing::debug;

pub enum StdInteratorSource {
    Args(VecDeque<String>),
    StdinRead(Box<dyn BufRead>),
}

pub struct StdInterator {
    source: StdInteratorSource,
}

impl StdInterator {
    pub fn new(input: Option<String>) -> Self {
        let source = if let Some(file_content) = input {
            if file_content == "-" {
                debug!("reading from stdin - got '-'");
                let reader = Box::new(io::BufReader::new(io::stdin()));
                StdInteratorSource::StdinRead(reader)
            } else {
                let mut lines = file_content.lines();
                if let Some(first_line) = lines.next() {
                    if lines.next().is_none() {
                        // Only one line in input
                        let filename = first_line;
                        if let Ok(file) = std::fs::File::open(filename) {
                            debug!("reading from file: {}", filename);
                            let reader = Box::new(io::BufReader::new(file));
                            StdInteratorSource::StdinRead(reader)
                        } else {
                            // Cannot open file, treat as single argument
                            debug!(
                                "could not open file: {}. Treating as args",
                                filename
                            );
                            let args = vec![filename.to_string()].into();
                            StdInteratorSource::Args(args)
                        }
                    } else {
                        // Multiple lines, treat each line as an argument
                        debug!("reading from args");
                        let args = file_content
                            .lines()
                            .map(ToString::to_string)
                            .collect::<VecDeque<String>>();
                        debug!("args: {:?}", args);
                        StdInteratorSource::Args(args)
                    }
                } else {
                    // Empty input, no arguments
                    debug!("empty input");
                    StdInteratorSource::Args(VecDeque::new())
                }
            }
        } else {
            let reader = Box::new(io::BufReader::new(io::stdin()));
            debug!("reading from stdin - no args");
            StdInteratorSource::StdinRead(reader)
        };
        Self { source }
    }
}

impl FromStr for StdInterator {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(StdInterator::new(Some(s.to_string())))
        // let source = if s == "-" {
        //     debug!("reading from stdin - got '-'");
        //     let reader = Box::new(io::BufReader::new(io::stdin()));
        //     StdInteratorSource::StdinRead(reader)
        // } else {
        //     debug!("reading from args");
        //     let args = s
        //         .lines() // This assumes that each line is separated by '\n'
        //         .map(std::string::ToString::to_string)
        //         .collect::<VecDeque<String>>();
        //     debug!("args: {:?}", args);
        //     StdInteratorSource::Args(args)
        // };
        // Ok(Self { source })
    }
}

impl Iterator for StdInterator {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            StdInteratorSource::Args(content) => content.pop_front().map(Ok),
            StdInteratorSource::StdinRead(reader) => {
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
