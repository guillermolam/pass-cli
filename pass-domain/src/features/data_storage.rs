use crate::{FolderKeyStorage, ShareKeyStorage};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait DataStorage: Send + Sync {
    async fn get_share_key_storage(&self) -> Arc<dyn ShareKeyStorage>;
    async fn get_folder_key_storage(&self) -> Arc<dyn FolderKeyStorage>;
}
