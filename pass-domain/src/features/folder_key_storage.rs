use crate::{DecryptedFolderKey, FolderId, ShareId};
use anyhow::Result;

#[async_trait::async_trait]
pub trait FolderKeyStorage: Send + Sync {
    async fn get_folder_keys(
        &self,
        share_id: &ShareId,
        folder_id: &FolderId,
    ) -> Result<Option<Vec<DecryptedFolderKey>>>;
    async fn store_folder_keys(
        &self,
        share_id: &ShareId,
        folder_id: &FolderId,
        folder_keys: Vec<DecryptedFolderKey>,
    ) -> Result<()>;
}
