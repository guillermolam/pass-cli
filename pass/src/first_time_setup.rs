use crate::PassClient;
use anyhow::{Context, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

const SERVICE_ACCOUNT_KEY_FILE_NAME: &str = "service_account_key";

#[derive(Zeroize, ZeroizeOnDrop)]
pub enum FirstTimeSetupKey {
    Passphrase(Vec<u8>),
    UserPassword(String),
    ServiceAccount(Vec<u8>),
}

impl PassClient {
    pub async fn perform_first_time_setup(&self, pass: &str) -> Result<()> {
        self.setup_key_passphrases(pass)
            .await
            .context("Error setting up key passphrases")?;

        Ok(())
    }

    pub async fn perform_first_time_setup_with_key(&self, key: FirstTimeSetupKey) -> Result<()> {
        match key {
            FirstTimeSetupKey::Passphrase(ref passphrase) => {
                self.setup_key_passphrases_with_passphrase(passphrase)
                    .await
                    .context("Error setting up key passphrases")?;
                Ok(())
            }
            FirstTimeSetupKey::UserPassword(ref user_pass) => {
                self.perform_first_time_setup(user_pass.as_str()).await
            }
            FirstTimeSetupKey::ServiceAccount(ref service_account_key) => {
                self.setup_service_account_key(service_account_key)
                    .await
                    .context("Error setting up service account key")?;
                Ok(())
            }
        }
    }

    async fn setup_service_account_key(&self, service_account_key: &[u8]) -> Result<()> {
        use std::path::Path;

        let local_key_provider = self.get_key_provider().await?;
        let local_key = local_key_provider.get_key().await?;

        // Encrypt the service account key with the local key
        let encrypted_key = pass_domain::crypto::encrypt(
            service_account_key,
            local_key.as_ref(),
            pass_domain::crypto::EncryptionTag::ServiceAccountKey,
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "Error encrypting service account key with local key: {:?}",
                e
            )
        })?;

        // Store the encrypted service account key
        let fs = self.client_features.get_fs().await;
        fs.store_file(encrypted_key, Path::new(SERVICE_ACCOUNT_KEY_FILE_NAME))
            .await
            .context("Error storing service account key")?;

        Ok(())
    }

    pub async fn get_local_service_account_key(&self) -> Result<Vec<u8>> {
        use std::path::Path;

        let fs = self.client_features.get_fs().await;
        let encrypted_key = fs
            .get_file(Path::new(SERVICE_ACCOUNT_KEY_FILE_NAME))
            .await
            .context("Error loading service account key")?;

        let local_key_provider = self.get_key_provider().await?;
        let local_key = local_key_provider.get_key().await?;

        let decrypted_key = pass_domain::crypto::decrypt(
            &encrypted_key,
            local_key.as_ref(),
            pass_domain::crypto::EncryptionTag::ServiceAccountKey,
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "Error decrypting service account key with local key: {:?}",
                e
            )
        })?;

        Ok(decrypted_key)
    }
}
