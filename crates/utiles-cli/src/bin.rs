mod cli;
mod lint;
mod shapes;
mod stdinterator;

#[tokio::main]
async fn main() {
    cli::cli_main(None, None)
}
