use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct Passphrase(pub(crate) Vec<u8>);

impl Passphrase {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for Passphrase {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, ZeroizeOnDrop)]
pub struct KeyPassphrase {
    pub id: String,
    pub passphrase: Passphrase,
}

#[derive(Clone, Debug)]
pub struct KeyPassphrases {
    passphrases: Vec<KeyPassphrase>,
}

impl KeyPassphrases {
    pub fn new(passphrases: Vec<KeyPassphrase>) -> KeyPassphrases {
        Self { passphrases }
    }

    pub fn into_map(self) -> HashMap<String, Passphrase> {
        let mut res = HashMap::new();
        for passphrase in self.passphrases {
            res.insert(passphrase.id.to_string(), passphrase.passphrase.clone());
        }
        res
    }
}
