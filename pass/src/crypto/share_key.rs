use crate::share::ShareKey;
use anyhow::Result;
use pass_domain::{PgpCrypto, PublicKey, UnlockedAddressKeys, UserKey};
use std::sync::Arc;

pub(crate) struct OpenShareKeyFlow {
    pub crypto: Arc<dyn PgpCrypto>,
    pub user_keys: Vec<UserKey>,
}

impl OpenShareKeyFlow {
    pub fn new(crypto: Arc<dyn PgpCrypto>, user_keys: Vec<UserKey>) -> Self {
        Self { crypto, user_keys }
    }

    pub async fn open(self, vault_key: ShareKey) -> Result<Vec<u8>> {
        let mut private_keys = vec![];
        let mut public_keys = vec![];

        for user_key in self.user_keys {
            let (private, public) = user_key.into_keys();
            private_keys.push(private);
            public_keys.push(public);
        }

        self.crypto
            .decrypt_and_verify(vault_key.key.0.clone(), private_keys, public_keys, None)
            .await
    }
}

pub(crate) struct OpenShareKeyForGroupFlow {
    pub crypto: Arc<dyn PgpCrypto>,
    pub address_keys: UnlockedAddressKeys,
    pub group_keys: Vec<PublicKey>,
}

impl OpenShareKeyForGroupFlow {
    pub fn new(
        crypto: Arc<dyn PgpCrypto>,
        address_keys: UnlockedAddressKeys,
        group_keys: Vec<PublicKey>,
    ) -> Self {
        Self {
            crypto,
            address_keys,
            group_keys,
        }
    }

    pub async fn open(self, vault_key: ShareKey) -> Result<Vec<u8>> {
        let private_keys = self.address_keys.into_keys();

        self.crypto
            .decrypt_and_verify(vault_key.key.0.clone(), private_keys, self.group_keys, None)
            .await
    }
}
