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
        // let source = match input {
        //     Some(file_content) => {
        //         if file_content == "-" {
        //             debug!("reading from stdin - got '-'");
        //             let reader = Box::new(io::BufReader::new(io::stdin()));
        //             StdInteratorSource::StdinRead(reader)
        //         } else {
        //             debug!("reading from args");
        //             let args = file_content
        //                 .lines() // This assumes that each line is separated by '\n'
        //                 .map(std::string::ToString::to_string)
        //                 .collect::<VecDeque<String>>();
        //             debug!("args: {:?}", args);
        //             StdInteratorSource::Args(args)
        //         }
        //     }
        //     None => {
        //         let reader = Box::new(io::BufReader::new(io::stdin()));
        //         debug!("reading from stdin - no args");
        //         StdInteratorSource::StdinRead(reader)
        //     }
        // };
        let source = if let Some(file_content) = input {
            if file_content == "-" {
                debug!("reading from stdin - got '-'");
                let reader = Box::new(io::BufReader::new(io::stdin()));
                StdInteratorSource::StdinRead(reader)
            } else {
                debug!("reading from args");
                let args = file_content
                    .lines() // This assumes that each line is separated by '\n'
                    .map(std::string::ToString::to_string)
                    .collect::<VecDeque<String>>();
                debug!("args: {:?}", args);
                StdInteratorSource::Args(args)
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
