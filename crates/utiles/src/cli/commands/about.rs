use crate::UtilesResult;

pub fn about_main() -> UtilesResult<()> {
    let current_exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    println!("{} ~ {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("desc:    {}", env!("CARGO_PKG_DESCRIPTION"));
    println!("repo:    {}", env!("CARGO_PKG_REPOSITORY"));
    println!("which:   {}", current_exe.display());
    println!(
        "profile: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    Ok(())
}
