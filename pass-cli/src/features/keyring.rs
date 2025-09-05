use anyhow::Context;
use keyring::{Entry, Error};
use pass_domain::LocalKeyProvider;
use tokio::sync::RwLock;

const KEYRING_SERVICE_NAME: &str = "ProtonPassCLI";
const KEYRING_CREDENTIAL_NAME: &str = "cli-local-key";

#[derive(Default)]
pub struct KeyringKeyProvider {
    key: RwLock<Option<Vec<u8>>>,
}

#[async_trait::async_trait]
impl LocalKeyProvider for KeyringKeyProvider {
    async fn get_key(&self) -> anyhow::Result<Vec<u8>> {
        let key_guard = self.key.read().await;
        if let Some(key) = &*key_guard {
            Ok(key.clone())
        } else {
            drop(key_guard);
            let mut write_key_guard = self.key.write().await;
            let key = get_local_key()
                .await
                .context("Could not get local key from keyring")?;
            *write_key_guard = Some(key.clone());
            Ok(key)
        }
    }
}

pub async fn get_local_key() -> anyhow::Result<Vec<u8>> {
    let entry = Entry::new(KEYRING_SERVICE_NAME, KEYRING_CREDENTIAL_NAME)
        .map_err(|e| anyhow::anyhow!("Error accessing credential: {e:?}"))?;

    match entry.get_secret() {
        Ok(cred) => Ok(cred),
        Err(e) => match e {
            Error::NoEntry => {
                info!("Credential not found in Keyring. Creating one");
                let cred = pass_domain::crypto::generate_encryption_key();
                entry
                    .set_secret(&cred)
                    .map_err(|e| anyhow::anyhow!("Error accessing keyring: {e}"))?;
                info!("Stored credential into keyring");
                Ok(cred)
            }
            _ => Err(anyhow::anyhow!(
                "Error accessing credential on keyring: {e:?}"
            )),
        },
    }
}
