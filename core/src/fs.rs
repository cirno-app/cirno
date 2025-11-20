use std::path::Path;
use std::task::Poll;

use anyhow::{Context, Result};
use tokio::fs;

pub async fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<u64> {
    fs::copy(&src, &dst).await.with_context(|| {
        format!(
            "Failed to copy file from {} to {}",
            src.as_ref().display(),
            dst.as_ref().display()
        )
    })
}

pub async fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&path)
        .await
        .with_context(|| format!("Failed to create directory: {}", path.as_ref().display()))
}

pub async fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    fs::create_dir(&path)
        .await
        .with_context(|| format!("Failed to create directory: {}", path.as_ref().display()))
}

pub async fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    fs::remove_dir_all(&path)
        .await
        .with_context(|| format!("Failed to remove directory: {}", path.as_ref().display()))
}

pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir<P>> {
    let inner = fs::read_dir(&path)
        .await
        .with_context(|| format!("Failed to read directory: {}", path.as_ref().display()))?;
    Ok(ReadDir { inner, path })
}

pub struct ReadDir<P: AsRef<Path>> {
    inner: fs::ReadDir,
    path: P,
}

impl<P: AsRef<Path>> ReadDir<P> {
    pub async fn next_entry(&mut self) -> Result<Option<fs::DirEntry>> {
        self.inner
            .next_entry()
            .await
            .with_context(|| format!("Failed to read directory: {}", self.path.as_ref().display()))
    }

    pub fn poll_next_entry(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<Option<fs::DirEntry>>> {
        self.inner
            .poll_next_entry(cx)
            .map(|result| result.with_context(|| format!("Failed to read directory: {}", self.path.as_ref().display())))
    }
}

pub async fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))
}

pub async fn remove_dir(path: impl AsRef<Path>) -> Result<()> {
    fs::remove_dir(&path)
        .await
        .with_context(|| format!("Failed to remove directory: {}", path.as_ref().display()))
}

pub async fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    fs::remove_file(&path)
        .await
        .with_context(|| format!("Failed to remove file: {}", path.as_ref().display()))
}

pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    fs::write(&path, contents)
        .await
        .with_context(|| format!("Failed to write file: {}", path.as_ref().display()))
}
