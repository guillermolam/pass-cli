use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug)]
pub enum PgpCryptoError {
    Unknown,
}

impl std::fmt::Display for PgpCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PgpCryptoError {}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey {
    content: Vec<u8>,
}

impl PrivateKey {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}

#[derive(Clone)]
pub struct PublicKey {
    content: Vec<u8>,
}

impl PublicKey {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PlainText(pub(crate) Vec<u8>);

impl PlainText {
    pub fn new(content: Vec<u8>) -> Self {
        Self(content)
    }
}

impl AsRef<[u8]> for PlainText {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub enum Signature {
    Bytes(Vec<u8>),
    Armored(String),
}

pub enum DataToDecrypt {
    RawData(Vec<u8>),
    DataWithSignature { data: Vec<u8>, signature: Signature },
}
