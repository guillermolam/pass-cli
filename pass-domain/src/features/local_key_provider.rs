use anyhow::Result;

#[async_trait::async_trait]
pub trait LocalKeyProvider {
    async fn get_key(&self) -> Result<Vec<u8>>;
}
