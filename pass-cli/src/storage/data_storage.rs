use pass_domain::{DataStorage, FolderKeyStorage, ShareKeyStorage};
use std::sync::Arc;

pub struct CliDataStorage {
    share_key_storage: Arc<dyn ShareKeyStorage>,
    folder_key_storage: Arc<dyn FolderKeyStorage>,
}

impl CliDataStorage {
    pub fn new(
        share_key_storage: Arc<dyn ShareKeyStorage>,
        folder_key_storage: Arc<dyn FolderKeyStorage>,
    ) -> Self {
        Self {
            share_key_storage,
            folder_key_storage,
        }
    }
}

#[async_trait::async_trait]
impl DataStorage for CliDataStorage {
    async fn get_share_key_storage(&self) -> Arc<dyn ShareKeyStorage> {
        self.share_key_storage.clone()
    }

    async fn get_folder_key_storage(&self) -> Arc<dyn FolderKeyStorage> {
        self.folder_key_storage.clone()
    }
}
