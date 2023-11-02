use std::io::BufRead;

use clap::{Args, Parser, Subcommand, ValueEnum};

mod cli;

// use crate::maybe_stdin::MaybeStdin;
// mod maybe_stdinput;

// use clap_stdin::MaybeStdin;

// use clap_stdin::MaybeStdin;

// use clap::Parser;
// use std::io::{self, BufRead, BufReader};
// use std::fs::File;
// use std::iter;

// #[derive(Parser, Debug)]
// struct Cli {
//     Input file, reads from stdin if not present
// #[clap(value_parser)]
// input: Option<String>,
// }
// impl From<tokio_rusqlite::Error> for Error {
//     fn from(e: tokio_rusqlite::Error) -> Error {
//         Error::RusqliteError(e)
//     }
// }

#[tokio::main]
async fn main() {
    cli::cli_main()
}
