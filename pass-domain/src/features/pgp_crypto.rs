use crate::{DataToDecrypt, Passphrase, PlainText, PrivateKey, PublicKey};
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum DataEncoding {
    Armored,
    Binary,
}

#[derive(Clone, Debug)]
pub enum DataToArmor {
    Message(Vec<u8>),
    Signature(Vec<u8>),
    PrivateKey(Vec<u8>),
    PublicKey(Vec<u8>),
}

#[async_trait::async_trait]
pub trait PgpCrypto {
    async fn encrypt(&self, data: Vec<u8>, key: PublicKey) -> Result<Vec<u8>>;
    async fn encrypt_and_sign(
        &self,
        data: PlainText,
        encryption_key: PublicKey,
        signing_key: PrivateKey,
        signing_context: Option<String>,
    ) -> Result<Vec<u8>>;

    async fn sign(&self, data: Vec<u8>, signing_key: PrivateKey) -> Result<Vec<u8>>;

    async fn decrypt(&self, data: Vec<u8>, keys: Vec<PrivateKey>) -> Result<Vec<u8>>;
    async fn decrypt_and_verify(
        &self,
        data: Vec<u8>,
        decryption_keys: Vec<PrivateKey>,
        verification_keys: Vec<PublicKey>,
        verification_context: Option<String>,
    ) -> Result<Vec<u8>>;
    async fn decrypt_and_verify_data(
        &self,
        data: DataToDecrypt,
        decryption_keys: Vec<PrivateKey>,
        verification_keys: Vec<PublicKey>,
        verification_context: Option<String>,
    ) -> Result<Vec<u8>>;

    async fn armor(&self, data: DataToArmor) -> Result<String>;
    async fn unarmor(&self, armored: String) -> Result<Vec<u8>>;

    async fn open_private_key(&self, key: PrivateKey, passphrase: Passphrase)
    -> Result<PrivateKey>;
    async fn get_public_key(&self, key: PrivateKey) -> Result<PublicKey>;
    async fn generate_key_pair(
        &self,
        name: String,
        email: String,
    ) -> Result<(PrivateKey, PublicKey)>;
}
