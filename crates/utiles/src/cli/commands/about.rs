use crate::UtilesResult;

pub fn about_main() -> UtilesResult<()> {
    println!("{} ~ {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("desc:    {}", env!("CARGO_PKG_DESCRIPTION"));
    println!("repo:    {}", env!("CARGO_PKG_REPOSITORY"));
    Ok(())
}
