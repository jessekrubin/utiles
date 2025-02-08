use crate::errors::UtilesResult;

pub(crate) fn about_main() -> UtilesResult<()> {
    let current_exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    let prof = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    // if (json){
    //     let j = serde_json::json! ({
    //         "name": env!("CARGO_PKG_NAME"),
    //         "version": env!("CARGO_PKG_VERSION"),
    //         "authors": env!("CARGO_PKG_AUTHORS"),
    //         "desc": env!("CARGO_PKG_DESCRIPTION"),
    //         "repo": env!("CARGO_PKG_REPOSITORY"),
    //         "which": current_exe.display().to_string(),
    //         "profile": prof,

    //     });
    //     println!("{}", serde_json::to_string_pretty(&j)?);
    //     return Ok(());
    // }

    let parts = [
        format!("{} ~ {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        format!("version: {}", env!("CARGO_PKG_VERSION")),
        format!("authors: {}", env!("CARGO_PKG_AUTHORS")),
        format!("desc:    {}", env!("CARGO_PKG_DESCRIPTION")),
        format!("repo:    {}", env!("CARGO_PKG_REPOSITORY")),
        format!("which:   {}", current_exe.display()),
        format!("profile: {prof}"),
    ];
    println!("{}", parts.join("\n"));
    Ok(())
}
