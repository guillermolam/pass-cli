use crate::{
    AddressKey, KeySalt, LockedUserKey, Passphrase, PrivateKey, PublicKey, UnlockedAddressKeys,
    UserKey,
};
use anyhow::Result;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait AccountCrypto {
    async fn generate_passphrases(
        &self,
        key_salts: Vec<KeySalt>,
        pass: &str,
    ) -> Result<HashMap<String, Passphrase>>;

    async fn open_user_keys(
        &self,
        keys: Vec<LockedUserKey>,
        passphrases: HashMap<String, Passphrase>,
    ) -> Result<Vec<UserKey>>;

    async fn open_address_keys(
        &self,
        user_keys: Vec<UserKey>,
        address_keys: Vec<AddressKey>,
    ) -> Result<UnlockedAddressKeys>;

    async fn open_address_keys_with_keys(
        &self,
        private_keys: Vec<PrivateKey>,
        public_keys: Vec<PublicKey>,
        address_keys: Vec<AddressKey>,
    ) -> Result<UnlockedAddressKeys>;
}
