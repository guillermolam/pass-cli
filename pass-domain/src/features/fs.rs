use anyhow::Result;
use std::path::Path;

#[async_trait::async_trait]
pub trait FsStorage {
    async fn get_file(&self, path: &Path) -> Result<Vec<u8>>;
    async fn file_exists(&self, path: &Path) -> Result<bool>;
    async fn store_file(&self, contents: Vec<u8>, path: &Path) -> Result<()>;
    async fn remove_file(&self, path: &Path) -> Result<()>;
}
