use tokio::fs;

use crate::errors::UtilesResult;
use crate::UtilesError;

pub async fn file_exists<P: AsRef<std::path::Path>>(p: P) -> bool {
    let metadata = fs::metadata(p).await;
    match metadata {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

// pub async fn dir_exists<P: AsRef<std::path::Path>>(p: P) -> bool {
//     let metadata = fs::metadata(p).await;
//     match metadata {
//         Ok(metadata) => metadata.is_dir(),
//         Err(_) => false,
//     }
// }

pub async fn file_exists_err<P: AsRef<std::path::Path>>(p: P) -> UtilesResult<bool> {
    if file_exists(&p).await {
        Ok(true)
    } else {
        let p_str = p.as_ref().to_string_lossy();
        Err(UtilesError::FileDoesNotExist(format!("{p_str:?}")))
    }
}

pub async fn filesize_async<P: AsRef<std::path::Path>>(p: P) -> Option<u64> {
    let metadata = fs::metadata(p).await.ok()?;
    Some(metadata.len())
}
