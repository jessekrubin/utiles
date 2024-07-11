use tokio::fs;

pub async fn file_exists<P: AsRef<std::path::Path>>(p: P) -> bool {
    let metadata = fs::metadata(p).await;
    match metadata {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

pub async fn filesize_async<P: AsRef<std::path::Path>>(p: P) -> Option<u64> {
    let metadata = fs::metadata(p).await.ok()?;
    Some(metadata.len())
}
