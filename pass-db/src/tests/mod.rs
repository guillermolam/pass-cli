use crate::DatabaseManager;
use anyhow::Result;
use pass_domain::LocalKey;

#[macro_export]
macro_rules! test_db {
    () => {{ create_test_db().await.expect("failed to create test db") }};
}

pub async fn create_test_db() -> Result<DatabaseManager> {
    create_test_db_with_key(LocalKey::new(vec![0u8; 32])).await
}

pub async fn create_test_db_with_key(encryption_key: LocalKey) -> Result<DatabaseManager> {
    DatabaseManager::new_in_memory(encryption_key).await
}
