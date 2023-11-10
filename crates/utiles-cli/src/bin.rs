mod cli;
mod shapes;
mod stdinterator;

#[tokio::main]
async fn main() {
    cli::cli_main(None, None)
}
