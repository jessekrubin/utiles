// mod args;
mod cli;
mod utilejson;
mod utilesqlite;
// mod commands;
// mod find;
// mod stdinterator;
// mod stdinterator_filter;

#[tokio::main]
async fn main() {
    cli::cli_main(None, None).await;
}
