mod cli;
mod stdinterator;

#[tokio::main]
async fn main() {
    cli::cli_main(Option::None, Option::None)
}
