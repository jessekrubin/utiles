use std::path::Path;

use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::UtilesError;
use crate::errors::UtilesResult;
use crate::sqlite::SqliteError;

pub async fn file_exists<P: AsRef<Path>>(p: P) -> bool {
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

pub async fn file_exists_err<P: AsRef<Path>>(p: P) -> UtilesResult<bool> {
    if file_exists(&p).await {
        Ok(true)
    } else {
        let p_str = p.as_ref().to_string_lossy();
        Err(UtilesError::FileDoesNotExist(format!("{p_str:?}")))
    }
}

pub async fn filesize_async<P: AsRef<Path>>(p: P) -> Option<u64> {
    let metadata = fs::metadata(p).await.ok()?;
    Some(metadata.len())
}

pub async fn read_nbytes<P, const N: usize>(p: P) -> UtilesResult<[u8; N]>
where
    P: AsRef<Path>,
{
    let mut file = fs::File::open(&p).await?;
    let mut buf = [0; N];
    file.read_exact(&mut buf).await.map_err(|_| {
        UtilesError::SqliteError(SqliteError::InvalidSqliteDb(
            p.as_ref().to_string_lossy().to_string(),
        ))
    })?;
    Ok(buf)
}
