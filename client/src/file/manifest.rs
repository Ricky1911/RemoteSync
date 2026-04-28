use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, task::JoinSet};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug)]
struct FileMetadata {
    hash: [u8; 32],
    modified: chrono::NaiveDateTime,
    size: u64,
}

impl FileMetadata {
    pub async fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        let hash = common::crypto::stream_hash(path).await?;
        let metadata = fs::metadata(path).await?;
        let modified = DateTime::<Utc>::from(metadata.modified()?).naive_local();
        let size = metadata.file_size();

        Ok(FileMetadata {
            hash,
            modified,
            size,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileManifest {
    manifest: HashMap<PathBuf, FileMetadata>,
}

impl FileManifest {
    pub async fn from_dir(src_path: &Path) -> anyhow::Result<Self> {
        let manifest = Arc::new(Mutex::new(HashMap::new()));
        let mut join_set = JoinSet::<Result<(), std::io::Error>>::new();
        let walker = WalkDir::new(src_path);
        let token = CancellationToken::new();
        for entry_result in walker.into_iter() {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    return Err(anyhow::Error::msg(format!(
                        "Error while traversing directory {src_path:?}: {e}"
                    )));
                }
            };
            let path = entry.path();
            let path_stripped = path.strip_prefix(src_path)?;
            if path.is_file() {
                let path = path.to_path_buf();
                let path_stripped = path_stripped.to_path_buf();
                let manifest = Arc::clone(&manifest);
                let token = token.clone();
                let task = async move {
                    let metadata = FileMetadata::from_file(&path.to_path_buf()).await?;
                    let mut guard = manifest.lock().await;
                    guard.insert(path_stripped.to_path_buf(), metadata);
                    Ok(())
                };
                join_set.spawn(async move {
                    tokio::select! {
                        _ = token.cancelled() => Ok(()),
                        result = task => result
                    }
                });
            };
        }
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(io_err)) => {
                    token.cancel();
                    return Err(io_err.into());
                }
                Err(join_err) => {
                    if join_err.is_panic() {
                        std::panic::resume_unwind(join_err.into_panic())
                    } else {
                        return Err(anyhow::Error::msg("Task cancelled"));
                    }
                }
            }
        }
        Ok(FileManifest {
            manifest: Arc::try_unwrap(manifest)
                .expect("Reference leaked")
                .into_inner(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn generate_file_manifest() {
        let path = std::env::current_dir().unwrap();
        let manifest = FileManifest::from_dir(&path).await.unwrap();
        println!("{manifest:?}");
    }
}
