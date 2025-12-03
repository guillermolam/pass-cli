use anyhow::Result;
use pass_db::{DatabaseManager, FolderKeyModel};
use pass_domain::{DecryptedFolderKey, FolderId, FolderKeyStorage, ShareId};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DatabaseFolderKeyStorage {
    db: DatabaseManager,
    user_id: Arc<RwLock<Option<String>>>,
}

impl DatabaseFolderKeyStorage {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db,
            user_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_user_id(&self, user_id: Option<String>) {
        *self.user_id.write().await = user_id;
    }
}

#[async_trait::async_trait]
impl FolderKeyStorage for DatabaseFolderKeyStorage {
    async fn get_folder_keys(
        &self,
        share_id: &ShareId,
        folder_id: &FolderId,
    ) -> Result<Option<Vec<DecryptedFolderKey>>> {
        let user_id = self.user_id.read().await.clone();

        let user_id = match user_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let models = FolderKeyModel::get_by_folder_id(
            &self.db,
            &user_id,
            share_id.value(),
            folder_id.value(),
        )
        .await?;

        if models.is_empty() {
            return Ok(None);
        }

        let keys = models
            .into_iter()
            .map(|model| DecryptedFolderKey::new(model.key_rotation, model.folder_key))
            .collect();

        Ok(Some(keys))
    }

    async fn store_folder_keys(
        &self,
        share_id: &ShareId,
        folder_id: &FolderId,
        folder_keys: Vec<DecryptedFolderKey>,
    ) -> Result<()> {
        let user_id = self.user_id.read().await.clone();

        let user_id = match user_id {
            Some(id) => id,
            None => {
                warn!("No user_id set, skipping folder key storage");
                return Ok(());
            }
        };

        for key in folder_keys {
            FolderKeyModel::insert(
                &self.db,
                &user_id,
                share_id.value(),
                folder_id.value(),
                key.key_rotation,
                key.key().to_vec(),
            )
            .await?;
        }

        Ok(())
    }
}
