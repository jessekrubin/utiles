use utiles::cli;

#[tokio::main]
async fn main() {
    let r = cli::cli_main(None).await;
    match r {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error -- {}", e);
            std::process::exit(1);
        }
    }
}
