use std::io::BufRead;

use clap::{Args, Parser, Subcommand, ValueEnum};

use tracing_subscriber;

mod cli;

#[tokio::main]
async fn main() {
    cli::cli_main(Option::None)
}
