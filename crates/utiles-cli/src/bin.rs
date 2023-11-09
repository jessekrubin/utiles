mod cli;
mod stdinterator;
mod shapes;

#[tokio::main]
async fn main() {
    cli::cli_main(Option::None, Option::None)
}
