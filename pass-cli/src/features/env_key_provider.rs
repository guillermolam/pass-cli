use anyhow::{Context, Result};
use pass_domain::utils::xor_key;
use pass_domain::{LocalKey, LocalKeyProvider};
use sha2::{Digest, Sha256};

const ENCRYPTION_KEY_ENV_VAR: &str = "PROTON_PASS_ENCRYPTION_KEY";

pub struct EnvLocalKeyProvider {
    xored_key: Vec<u8>,
    xor_byte: u8,
}

impl EnvLocalKeyProvider {
    pub fn new() -> Result<Self> {
        let key_value = std::env::var(ENCRYPTION_KEY_ENV_VAR)
            .context(format!("{ENCRYPTION_KEY_ENV_VAR} environment variable must be set and non-empty when using env key provider"))?;

        if key_value.is_empty() {
            return Err(anyhow::anyhow!(
                "{ENCRYPTION_KEY_ENV_VAR} environment variable must be set and non-empty when using env key provider"
            ));
        }

        let mut hasher = Sha256::new();
        hasher.update(key_value.as_bytes());
        let hashed_key = hasher.finalize().to_vec();

        let xor_byte = pass_domain::crypto::generate_random_byte();

        let xored_key = xor_key(&hashed_key, xor_byte);

        Ok(Self {
            xored_key,
            xor_byte,
        })
    }
}

#[async_trait::async_trait]
impl LocalKeyProvider for EnvLocalKeyProvider {
    async fn get_key(&self) -> Result<LocalKey> {
        let key = xor_key(&self.xored_key, self.xor_byte);
        Ok(LocalKey::new(key))
    }

    async fn remove_key(&self) -> Result<()> {
        // Nothing to remove since the key only lives in process memory
        Ok(())
    }
}
