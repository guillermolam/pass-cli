#[cfg(feature = "keyring-provider")]
pub(crate) mod keyring;

use crate::storage::get_local_key;
use anyhow::Result;
use pass_domain::{AccountCrypto, ClientFeatures, FsStorage, LocalKeyProvider};
use pass_fs::RealFsStorage;
use pass_pgp::{NativePgpCrypto, ProtonAccountCrypto};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(feature = "keyring-provider"))]
fn get_key_provider(base_dir: PathBuf) -> Arc<dyn LocalKeyProvider + Send + Sync> {
    Arc::new(FsLocalKeyProvider::new(base_dir))
}

#[cfg(feature = "keyring-provider")]
fn get_key_provider(_base_dir: PathBuf) -> Arc<dyn LocalKeyProvider + Send + Sync> {
    Arc::new(keyring::KeyringKeyProvider::default())
}

#[derive(Clone)]
pub struct CliClientFeatures {
    pub storage: Arc<RealFsStorage>,
    pub key_provider: Arc<dyn LocalKeyProvider + Send + Sync>,
}

impl CliClientFeatures {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            storage: Arc::new(RealFsStorage::new(base_dir.clone())),
            key_provider: get_key_provider(base_dir),
        }
    }
}

#[derive(Clone)]
pub struct FsLocalKeyProvider {
    base_dir: PathBuf,
}

impl FsLocalKeyProvider {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

#[async_trait::async_trait]
impl LocalKeyProvider for FsLocalKeyProvider {
    async fn get_key(&self) -> Result<Vec<u8>> {
        get_local_key(&self.base_dir).await
    }
}

#[async_trait::async_trait]
impl ClientFeatures for CliClientFeatures {
    async fn get_local_key_provider(&self) -> Result<Arc<dyn LocalKeyProvider>> {
        Ok(self.key_provider.clone())
    }

    async fn get_account_crypto(&self) -> Arc<dyn AccountCrypto> {
        Arc::new(ProtonAccountCrypto)
    }

    async fn get_fs(&self) -> Arc<dyn FsStorage> {
        self.storage.clone()
    }

    async fn get_pgp_crypto(&self) -> Arc<dyn pass_domain::PgpCrypto> {
        Arc::new(NativePgpCrypto)
    }
}
