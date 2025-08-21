use crate::{
    ApiKey, ApiKeySalt, ClientFeatures, Passphrase, PgpCrypto, PrivateKey, PublicKey,
    UnlockedAddressKeys, UserKey,
};
use anyhow::Result;
use pass_domain::AddressKey;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct TestClientFeatures {
    base_dir: PathBuf,
}

impl TestClientFeatures {
    pub fn new() -> Self {
        let base_dir = tempdir::TempDir::new("pass_cli_test").expect("Couldn't create temp dir");
        Self {
            base_dir: base_dir.into_path(),
        }
    }
}

#[async_trait::async_trait]
impl ClientFeatures for TestClientFeatures {
    async fn get_local_key(&self) -> Result<Vec<u8>> {
        todo!()
    }

    async fn get_file(&self, path: &Path) -> Result<Vec<u8>> {
        todo!()
    }

    async fn file_exists(&self, path: &Path) -> Result<bool> {
        todo!()
    }

    async fn store_file(&self, contents: Vec<u8>, path: &Path) -> Result<()> {
        todo!()
    }

    async fn remove_file(&self, path: &Path) -> Result<()> {
        todo!()
    }

    async fn generate_passphrases(
        &self,
        key_salts: Vec<ApiKeySalt>,
        pass: &str,
    ) -> Result<HashMap<String, Passphrase>> {
        todo!()
    }

    async fn open_user_keys(
        &self,
        keys: Vec<ApiKey>,
        passphrases: HashMap<String, Passphrase>,
    ) -> Result<Vec<UserKey>> {
        todo!()
    }

    async fn open_address_keys(
        &self,
        user_keys: Vec<UserKey>,
        address_keys: Vec<AddressKey>,
    ) -> Result<UnlockedAddressKeys> {
        todo!()
    }

    async fn open_address_keys_with_keys(
        &self,
        private_keys: Vec<PrivateKey>,
        public_keys: Vec<PublicKey>,
        address_keys: Vec<AddressKey>,
    ) -> Result<UnlockedAddressKeys> {
        todo!()
    }

    async fn get_pgp_crypto(&self) -> Arc<dyn PgpCrypto> {
        todo!()
    }
}
