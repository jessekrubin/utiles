use utiles::cli;

#[tokio::main]
async fn main() {
    cli::cli_main(None).await.unwrap();
}
