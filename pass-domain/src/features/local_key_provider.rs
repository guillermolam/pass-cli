use anyhow::Result;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct LocalKey(Vec<u8>);

impl LocalKey {
    pub fn new(key: Vec<u8>) -> Self {
        Self(key)
    }
}

impl AsRef<[u8]> for LocalKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[async_trait::async_trait]
pub trait LocalKeyProvider: Send + Sync {
    async fn get_key(&self) -> Result<LocalKey>;
    async fn remove_key(&self) -> Result<()>;
}
