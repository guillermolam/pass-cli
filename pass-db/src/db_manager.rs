use deadpool::managed::{Manager, Metrics, Object, RecycleResult};
use pass_domain::LocalKey;
use pass_domain::utils::xor_key;
use std::ops::Deref;

pub fn format_key_for_sqlcipher(key: &[u8]) -> String {
    let key_hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();

    // Format for SQLCipher to use literal bytes: x'BYTES_IN_HEX_HERE'
    format!("x'{}'", key_hex)
}

pub struct EncryptedSqliteManager {
    path: String,
    encryption_key: Vec<u8>,
    xor_key: u8,
}

impl EncryptedSqliteManager {
    pub fn new(path: String, encryption_key: LocalKey) -> Self {
        let xor_key_byte = pass_domain::crypto::generate_random_byte();
        let xored_key = xor_key(encryption_key.as_ref(), xor_key_byte);
        Self {
            path,
            encryption_key: xored_key,
            xor_key: xor_key_byte,
        }
    }

    fn get_sqlcipher_key(&self) -> String {
        let raw_value = xor_key(&self.encryption_key, self.xor_key);
        format_key_for_sqlcipher(&raw_value)
    }
}

impl Manager for EncryptedSqliteManager {
    type Type = rusqlite::Connection;
    type Error = rusqlite::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let path = self.path.clone();
        let key = self.get_sqlcipher_key();

        tokio::task::spawn_blocking(move || {
            let conn = rusqlite::Connection::open(&path)?;
            // Set SQLCipher encryption key immediately after opening
            // Use pragma_update instead of execute because PRAGMA key returns results
            conn.pragma_update(None, "key", &key)?;
            // Verify the key is correct by querying the database
            let _ = conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()));
            Ok(conn)
        })
        .await
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        conn.execute("SELECT 1", [])
            .map(|_| ())
            .map_err(deadpool::managed::RecycleError::Backend)
    }
}

pub struct DbConnection {
    pub obj: Object<EncryptedSqliteManager>,
}

impl DbConnection {
    /// Execute a closure with access to the database connection in a blocking context
    pub fn interact<F, R>(&self, func: F) -> impl Future<Output = anyhow::Result<R>> + '_
    where
        F: FnOnce(&rusqlite::Connection) -> R + Send,
        R: Send + 'static,
    {
        let conn: &rusqlite::Connection = self.obj.deref();

        // Use block_in_place to run blocking code without moving to a separate thread
        // This executes synchronously and returns a ready future
        // Wrap the result in Ok to create the outer Result layer
        std::future::ready(Ok(tokio::task::block_in_place(|| func(conn))))
    }
}
