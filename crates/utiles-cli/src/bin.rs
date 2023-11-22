mod args;
mod cli;
mod commands;
mod find;
mod lint;
mod shapes;
mod stdinterator;
mod stdinterator_filter;

#[tokio::main]
async fn main() {
    cli::cli_main(None, None).await;
}
