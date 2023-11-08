// use std::io;
// use std::io::BufRead;
// use clap::builder::Str;
// use tracing::debug;
//
// pub enum StdInteratorSource {
//     Args(Vec<&str>),
//     StdinRead(Box<dyn BufRead>),
// }
//
// pub struct StdInterator {
//     source: StdInteratorSource,
// }
//
// impl StdInterator {
//     pub fn new(input: Option<String>) -> io::Result<Self> {
//         let source = match input {
//             Some(file_content) => {
//                 if file_content == "-" {
//                     debug!("reading from stdin - got '-'");
//                     let reader = Box::new(io::BufReader::new(io::stdin()));
//                     StdInteratorSource::StdinRead(reader)
//                 } else {
//                     debug!("reading from args: {:?}", file_content);
//                     StdInteratorSource::Args(
//                         file_content.splitn(
//                             file_content.matches(char::is_whitespace).count() + 1,
//                             char::is_whitespace,
//                         ).collect::<Vec<&str>>()
//                         // ).map(|s| s.to_string()).collect(),
//                     )
//                 }
//             }
//             None => {
//                 let reader = Box::new(io::BufReader::new(io::stdin()));
//                 debug!("reading from stdin - no args");
//                 StdInteratorSource::StdinRead(reader)
//             }
//         };
//         Ok(Self { source })
//     }
// }
//
// impl Iterator for StdInterator {
//     type Item = io::Result< >;
//     fn next(&mut self) -> Option<Self::Item> {
//         match &mut self.source {
//             StdInteratorSource::Args(content) => {
//
//             }
//             StdInteratorSource::StdinRead(reader) => {
//                 let mut line = String::new();
//                 match reader.read_line(&mut line) {
//                     Ok(0) => None, // EOF
//                     Ok(_) => {
//                         let l
//                         Some(Ok(line.trim_end().to_string()))
//                     },
//                     Err(e) => Some(Err(e)),
//                 }
//             }
//         }
//     }
// }
use std::io;
use std::io::BufRead;
use std::collections::VecDeque;
use tracing::debug;

pub enum StdInteratorSource {
    Args(VecDeque<String>),
    StdinRead(Box<dyn BufRead>),
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
                    StdInteratorSource::StdinRead(reader)
                } else {
                    debug!("reading from args");
                    let args = file_content
                        .lines() // This assumes that each line is separated by '\n'
                        .map(|s| s.to_string())
                        .collect::<VecDeque<String>>();
                    StdInteratorSource::Args(args)
                }
            }
            None => {
                let reader = Box::new(io::BufReader::new(io::stdin()));
                debug!("reading from stdin - no args");
                StdInteratorSource::StdinRead(reader)
            }
        };
        Ok(Self { source })
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
