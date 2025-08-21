use anyhow::Result;
use pass_domain::{AccountCrypto, ClientFeatures, FsStorage, LocalKeyProvider, PgpCrypto};
use pass_fs::RealFsStorage;
use pass_pgp::{NativePgpCrypto, ProtonAccountCrypto};
use std::path::Path;
use std::sync::Arc;

pub struct StaticKeyProvider {
    pub key: Vec<u8>,
}

#[async_trait::async_trait]
impl LocalKeyProvider for StaticKeyProvider {
    async fn get_key(&self) -> Result<Vec<u8>> {
        Ok(self.key.clone())
    }
}

pub struct TempFsStorage {
    pub _dir: tempdir::TempDir,
    pub storage: RealFsStorage,
}

impl TempFsStorage {
    pub fn new(dir: tempdir::TempDir) -> Self {
        let path = dir.path().to_path_buf();
        Self {
            _dir: dir,
            storage: RealFsStorage::new(path),
        }
    }
}

#[async_trait::async_trait]
impl FsStorage for TempFsStorage {
    async fn get_file(&self, path: &Path) -> Result<Vec<u8>> {
        self.storage.get_file(path).await
    }

    async fn file_exists(&self, path: &Path) -> Result<bool> {
        self.storage.file_exists(path).await
    }

    async fn store_file(&self, contents: Vec<u8>, path: &Path) -> Result<()> {
        self.storage.store_file(contents, path).await
    }

    async fn remove_file(&self, path: &Path) -> Result<()> {
        self.storage.remove_file(path).await
    }
}

#[derive(Clone)]
pub struct TestClientFeatures {
    pub storage: Arc<TempFsStorage>,
    pub key_provider: Arc<StaticKeyProvider>,
}

impl TestClientFeatures {
    pub fn new(key: Vec<u8>) -> Self {
        let temp_dir =
            tempdir::TempDir::new("pass_test_features").expect("Failed to create temp dir");
        Self {
            storage: Arc::new(TempFsStorage::new(temp_dir)),
            key_provider: Arc::new(StaticKeyProvider { key }),
        }
    }
}

#[async_trait::async_trait]
impl ClientFeatures for TestClientFeatures {
    async fn get_local_key_provider(&self) -> Result<Arc<dyn LocalKeyProvider>> {
        Ok(self.key_provider.clone())
    }

    async fn get_account_crypto(&self) -> Arc<dyn AccountCrypto> {
        Arc::new(ProtonAccountCrypto)
    }

    async fn get_fs(&self) -> Arc<dyn FsStorage> {
        self.storage.clone()
    }

    async fn get_pgp_crypto(&self) -> Arc<dyn PgpCrypto> {
        Arc::new(NativePgpCrypto)
    }
}
