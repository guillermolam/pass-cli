use anyhow::{Context, Result, anyhow, bail};
use pass::PassClient;
use pass_domain::{Item, ItemContent, ShareId};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use ssh_agent_lib::agent::{ListeningSocket, Session, listen};
use ssh_agent_lib::error::AgentError;
use ssh_agent_lib::proto::extension::{QueryResponse, SessionBind};
use ssh_agent_lib::proto::{
    AddIdentity, AddIdentityConstrained, AddSmartcardKeyConstrained, Credential, Extension,
    RemoveIdentity, SignRequest, SmartcardKey, message,
};
use ssh_key::{
    Algorithm, HashAlg, Signature,
    private::{KeypairData, PrivateKey as SshPrivateKey},
    public::PublicKey as SshPublicKey,
};

#[cfg(windows)]
use ssh_agent_lib::agent::NamedPipeListener;
#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use rsa::pkcs1v15::SigningKey;
use rsa::sha2::{Sha256, Sha512};
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use sha1::Sha1;

#[derive(Clone)]
pub enum VaultQuery {
    ShareId(ShareId),
    VaultName(String),
    All,
}

impl VaultQuery {
    pub fn new(share_id: Option<String>, vault_name: Option<String>) -> Result<Self> {
        match (share_id, vault_name) {
            (Some(share_id), None) => Ok(Self::ShareId(ShareId::new(share_id))),
            (None, Some(vault_name)) => Ok(Self::VaultName(vault_name)),
            (None, None) => Ok(Self::All),
            (Some(_), Some(_)) => Err(anyhow!(
                "Please provide either --share-id or --vault-name, not both"
            )),
        }
    }
}

#[cfg(unix)]
fn get_default_socket_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(home_dir.join(".ssh").join("proton-pass-agent.sock"))
}

#[cfg(windows)]
fn get_default_socket_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    // On Windows, we'll use the path for reference, but actual pipe name is different
    Ok(home_dir.join(".ssh").join("proton-pass-agent"))
}

async fn load_ssh_keys_from_vaults(
    client: &PassClient,
    query: VaultQuery,
) -> Result<Vec<(Item, String, String)>> {
    let mut all_keys = Vec::new();

    match query {
        VaultQuery::ShareId(share_id) => {
            let items = client
                .list_items(&share_id)
                .await
                .context("Error listing items")?;
            all_keys.extend(extract_ssh_keys(items));
        }
        VaultQuery::VaultName(vault_name) => {
            let vault = client
                .find_vault(&vault_name)
                .await
                .context("Error finding vault")?;
            let items = client
                .list_items(&vault.share_id)
                .await
                .context("Error listing items")?;
            all_keys.extend(extract_ssh_keys(items));
        }
        VaultQuery::All => {
            let vaults = client.list_vaults().await.context("Error listing vaults")?;
            for vault in vaults {
                let items = client.list_items(&vault.share_id).await.context(format!(
                    "Error listing items for vault {}",
                    vault.content.name
                ))?;
                all_keys.extend(extract_ssh_keys(items));
            }
        }
    }

    Ok(all_keys)
}

fn extract_ssh_keys(items: Vec<Item>) -> Vec<(Item, String, String)> {
    items
        .into_iter()
        .filter_map(|item| {
            if let ItemContent::SshKey(ref ssh_key) = item.content.content {
                Some((
                    item.clone(),
                    ssh_key.private_key.clone(),
                    ssh_key.public_key.clone(),
                ))
            } else {
                None
            }
        })
        .collect()
}

fn find_passphrase_in_extra_fields(item: &Item) -> Option<String> {
    // Search terms to look for in field names (case-insensitive, partial match)
    let search_terms = [
        "passphrase",
        "password",
        "pass",
        "pwd",
        "key password",
        "ssh pass",
        "ssh password",
        "key pass",
    ];

    for extra_field in &item.content.extra_fields {
        let field_name_lower = extra_field.name.to_lowercase();

        // Check if any search term is contained in the field name
        for term in &search_terms {
            if field_name_lower.contains(term) {
                // Extract the content based on field type
                let content = match &extra_field.content {
                    pass_domain::ItemExtraFieldContent::Text(s) => Some(s.clone()),
                    pass_domain::ItemExtraFieldContent::Hidden(s) => Some(s.clone()),
                    pass_domain::ItemExtraFieldContent::Totp(_) => None,
                    pass_domain::ItemExtraFieldContent::Timestamp(_) => None,
                };

                if let Some(passphrase) = content
                    && !passphrase.is_empty()
                {
                    debug!(
                        "Found passphrase in field '{}' for item '{}'",
                        extra_field.name, item.content.title
                    );
                    return Some(passphrase);
                }
            }
        }
    }

    // If no extra field with that name is found, try to find the first one of type hidden
    for extra_field in &item.content.extra_fields {
        if let pass_domain::ItemExtraFieldContent::Hidden(ref val) = extra_field.content
            && !val.is_empty() {
                debug!(
                    "Best effort guess for passphrase in field '{}' for item '{}'",
                    extra_field.name, item.content.title
                );
                return Some(val.to_string());
            }
    }

    None
}

fn load_and_decrypt_key(item: &Item, private_key_str: &str) -> Result<SshPrivateKey> {
    let private_key = SshPrivateKey::from_openssh(private_key_str).context(format!(
        "Failed to parse SSH private key for item '{}'",
        item.content.title
    ))?;

    if !private_key.is_encrypted() {
        return Ok(private_key);
    }

    debug!(
        "Key '{}' is encrypted, looking for passphrase",
        item.content.title
    );

    if let Some(passphrase) = find_passphrase_in_extra_fields(item) {
        debug!(
            "Attempting to decrypt key '{}' with found passphrase",
            item.content.title
        );

        let decrypted = private_key.decrypt(passphrase).context(format!(
            "Failed to decrypt SSH key '{}' with provided passphrase",
            item.content.title
        ))?;

        info!("Successfully decrypted SSH key '{}'", item.content.title);
        Ok(decrypted)
    } else {
        Err(anyhow!(
            "SSH key '{}' is encrypted but no passphrase found in extra fields. \
            Please add a Hidden field named 'Passphrase' or 'Password' with the key's passphrase.",
            item.content.title
        ))
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Identity {
    public_key: SshPublicKey,
    private_key: SshPrivateKey,
    comment: String,
}

#[derive(Default, Clone)]
struct KeyStorage {
    identities: Arc<Mutex<Vec<Identity>>>,
}

impl KeyStorage {
    async fn identity_from_pubkey(&self, pubkey: &SshPublicKey) -> Option<Identity> {
        let identities = self.identities.lock().await;

        let index = Self::identity_index_from_pubkey(&identities, pubkey)?;
        Some(identities[index].clone())
    }

    async fn identity_add(&self, identity: Identity) {
        let mut identities = self.identities.lock().await;
        if Self::identity_index_from_pubkey(&identities, &identity.public_key).is_none() {
            identities.push(identity);
        }
    }

    async fn identity_remove(&self, pubkey: &SshPublicKey) -> Result<(), AgentError> {
        let mut identities = self.identities.lock().await;

        if let Some(index) = Self::identity_index_from_pubkey(&identities, pubkey) {
            identities.remove(index);
            Ok(())
        } else {
            Err(std::io::Error::other("Failed to remove identity: identity not found").into())
        }
    }

    async fn replace_all_identities(&self, new_identities: Vec<Identity>) {
        let mut identities = self.identities.lock().await;
        *identities = new_identities;
    }

    fn identity_index_from_pubkey(identities: &[Identity], pubkey: &SshPublicKey) -> Option<usize> {
        // Compare by key data instead of the full PublicKey object, since metadata might differ
        let target_key_data = pubkey.key_data();
        for (index, identity) in identities.iter().enumerate() {
            if identity.public_key.key_data() == target_key_data {
                return Some(index);
            }
        }
        None
    }
}

#[ssh_agent_lib::async_trait]
impl Session for KeyStorage {
    async fn request_identities(&mut self) -> Result<Vec<message::Identity>, AgentError> {
        let mut identities = vec![];
        for identity in self.identities.lock().await.iter() {
            identities.push(message::Identity {
                pubkey: identity.public_key.key_data().clone(),
                comment: identity.comment.clone(),
            })
        }
        Ok(identities)
    }

    async fn sign(&mut self, sign_request: SignRequest) -> Result<Signature, AgentError> {
        let pubkey: SshPublicKey = sign_request.pubkey.clone().into();

        debug!(
            "Sign request for public key: {:?}",
            pubkey.fingerprint(HashAlg::Sha256)
        );

        // Log all available identities for debugging
        {
            let identities = self.identities.lock().await;
            debug!("Available identities: {}", identities.len());
            for (idx, id) in identities.iter().enumerate() {
                debug!(
                    "  Identity {}: {} - {:?}",
                    idx,
                    id.comment,
                    id.public_key.fingerprint(HashAlg::Sha256)
                );
            }
        }

        if let Some(identity) = self.identity_from_pubkey(&pubkey).await {
            debug!("Found matching identity: {}", identity.comment);
            match identity.private_key.key_data() {
                KeypairData::Rsa(key) => {
                    let algorithm;

                    let private_key: rsa::RsaPrivateKey =
                        key.try_into().map_err(AgentError::other)?;
                    let mut rng = rand::thread_rng();
                    let data = &sign_request.data;

                    let signature = if sign_request.flags
                        & ssh_agent_lib::proto::signature::RSA_SHA2_512
                        != 0
                    {
                        algorithm = "rsa-sha2-512";
                        SigningKey::<Sha512>::new(private_key).sign_with_rng(&mut rng, data)
                    } else if sign_request.flags & ssh_agent_lib::proto::signature::RSA_SHA2_256
                        != 0
                    {
                        algorithm = "rsa-sha2-256";
                        SigningKey::<Sha256>::new(private_key).sign_with_rng(&mut rng, data)
                    } else {
                        algorithm = "ssh-rsa";
                        SigningKey::<Sha1>::new(private_key).sign_with_rng(&mut rng, data)
                    };
                    Ok(Signature::new(
                        Algorithm::new(algorithm).map_err(AgentError::other)?,
                        signature.to_bytes().to_vec(),
                    )
                    .map_err(AgentError::other)?)
                }
                KeypairData::Ed25519(key) => {
                    use ed25519_dalek::{Signer, SigningKey as Ed25519SigningKey};
                    let signing_key = Ed25519SigningKey::from_bytes(&key.private.to_bytes());
                    let signature_bytes: ed25519_dalek::Signature =
                        signing_key.sign(&sign_request.data);

                    Ok(Signature::new(
                        Algorithm::new("ssh-ed25519").map_err(AgentError::other)?,
                        signature_bytes.to_bytes().to_vec(),
                    )
                    .map_err(AgentError::other)?)
                }
                KeypairData::Ecdsa(keypair) => {
                    use ssh_key::EcdsaCurve;

                    let (algorithm, signature_bytes) = match keypair.curve() {
                        EcdsaCurve::NistP256 => {
                            use p256::ecdsa::{SigningKey, signature::Signer};
                            use p256::elliptic_curve::generic_array::GenericArray;
                            let private_bytes = keypair.private_key_bytes();
                            let key_array = GenericArray::from_slice(private_bytes);
                            let signing_key =
                                SigningKey::from_bytes(key_array).map_err(AgentError::other)?;
                            let sig: p256::ecdsa::Signature = signing_key.sign(&sign_request.data);
                            ("ecdsa-sha2-nistp256", sig.to_bytes().to_vec())
                        }
                        EcdsaCurve::NistP384 => {
                            use p384::ecdsa::{SigningKey, signature::Signer};
                            use p384::elliptic_curve::generic_array::GenericArray;
                            let private_bytes = keypair.private_key_bytes();
                            let key_array = GenericArray::from_slice(private_bytes);
                            let signing_key =
                                SigningKey::from_bytes(key_array).map_err(AgentError::other)?;
                            let sig: p384::ecdsa::Signature = signing_key.sign(&sign_request.data);
                            ("ecdsa-sha2-nistp384", sig.to_bytes().to_vec())
                        }
                        EcdsaCurve::NistP521 => {
                            use p521::ecdsa::{SigningKey, signature::Signer};
                            use p521::elliptic_curve::generic_array::GenericArray;
                            let private_bytes = keypair.private_key_bytes();
                            let key_array = GenericArray::from_slice(private_bytes);
                            let signing_key =
                                SigningKey::from_bytes(key_array).map_err(AgentError::other)?;
                            let sig: p521::ecdsa::Signature = signing_key.sign(&sign_request.data);
                            ("ecdsa-sha2-nistp521", sig.to_bytes().to_vec())
                        }
                    };

                    Ok(Signature::new(
                        Algorithm::new(algorithm).map_err(AgentError::other)?,
                        signature_bytes,
                    )
                    .map_err(AgentError::other)?)
                }
                _ => Err(std::io::Error::other("Signature for key type not implemented").into()),
            }
        } else {
            error!("Failed to find identity for requested public key");
            Err(std::io::Error::other("Failed to create signature: identity not found").into())
        }
    }

    async fn add_identity(&mut self, identity: AddIdentity) -> Result<(), AgentError> {
        if let Credential::Key { privkey, comment } = identity.credential {
            let privkey = SshPrivateKey::try_from(privkey).map_err(AgentError::other)?;
            self.identity_add(Identity {
                public_key: SshPublicKey::from(&privkey),
                private_key: privkey,
                comment,
            })
            .await;
            Ok(())
        } else {
            info!("Unsupported key type: {:#?}", identity.credential);
            Ok(())
        }
    }

    async fn add_identity_constrained(
        &mut self,
        identity: AddIdentityConstrained,
    ) -> Result<(), AgentError> {
        let AddIdentityConstrained {
            identity,
            constraints,
        } = identity;
        info!("Would use these constraints: {constraints:#?}");
        self.add_identity(identity).await
    }

    async fn remove_identity(&mut self, identity: RemoveIdentity) -> Result<(), AgentError> {
        let pubkey: SshPublicKey = identity.pubkey.into();
        self.identity_remove(&pubkey).await?;
        Ok(())
    }

    async fn add_smartcard_key(&mut self, key: SmartcardKey) -> Result<(), AgentError> {
        info!("Adding smartcard key: {key:?}");
        Ok(())
    }

    async fn add_smartcard_key_constrained(
        &mut self,
        key: AddSmartcardKeyConstrained,
    ) -> Result<(), AgentError> {
        info!("Adding smartcard key with constraints: {key:?}");
        Ok(())
    }

    async fn lock(&mut self, pwd: String) -> Result<(), AgentError> {
        info!("Locked with password: {pwd:?}");
        Ok(())
    }

    async fn unlock(&mut self, pwd: String) -> Result<(), AgentError> {
        info!("Unlocked with password: {pwd:?}");
        Ok(())
    }

    async fn extension(&mut self, extension: Extension) -> Result<Option<Extension>, AgentError> {
        info!("Extension request: {}", extension.name);

        match extension.name.as_str() {
            "query" => {
                let response = Extension::new_message(QueryResponse {
                    extensions: vec!["query".into(), "session-bind@openssh.com".into()],
                })?;
                Ok(Some(response))
            }
            "session-bind@openssh.com" => {
                match extension.parse_message::<SessionBind>()? {
                    Some(bind) => {
                        // Verify the session binding signature
                        bind.verify_signature()
                            .map_err(|_| AgentError::ExtensionFailure)?;

                        info!("Session binding verified successfully");
                        Ok(None)
                    }
                    None => {
                        warn!("Failed to parse session-bind extension");
                        Err(AgentError::Failure)
                    }
                }
            }
            _ => {
                info!("Unsupported extension: {}", extension.name);
                Err(AgentError::Failure)
            }
        }
    }
}

async fn load_keys_into_storage(
    client: &PassClient,
    vault_query: &VaultQuery,
) -> Result<Vec<Identity>> {
    let ssh_key_items = load_ssh_keys_from_vaults(client, vault_query.clone())
        .await
        .context("Failed to load SSH keys from vaults")?;

    if ssh_key_items.is_empty() {
        return Ok(Vec::new());
    }

    let mut identities = Vec::new();

    for (item, private_key_str, _public_key_str) in ssh_key_items {
        match load_and_decrypt_key(&item, &private_key_str) {
            Ok(private_key) => {
                let public_key = SshPublicKey::from(&private_key);
                identities.push(Identity {
                    comment: item.content.title.clone(),
                    private_key,
                    public_key,
                });
            }
            Err(e) => {
                warn!("Failed to load key '{}': {}", item.content.title, e);
            }
        }
    }

    Ok(identities)
}

async fn refresh_keys_periodically(
    client: &PassClient,
    vault_query: &VaultQuery,
    key_storage: &KeyStorage,
    _interval_secs: u64,
) {
    info!("Refreshing SSH keys from Proton Pass...");

    match load_keys_into_storage(client, vault_query).await {
        Ok(identities) => {
            let count = identities.len();
            key_storage.replace_all_identities(identities).await;
            info!("Refreshed {} SSH key(s)", count);
        }
        Err(e) => {
            warn!("Failed to refresh SSH keys: {}. Will retry later.", e);
        }
    }
}

fn print_agent_startup_message(socket_display: &str, refresh_interval: u64) {
    eprintln!("SSH agent started successfully!");
    eprintln!("To use this agent, run:");
    #[cfg(unix)]
    eprintln!("  export SSH_AUTH_SOCK={}", socket_display);
    #[cfg(windows)]
    eprintln!("  $env:SSH_AUTH_SOCK = '{}'", socket_display);

    if refresh_interval > 0 {
        eprintln!(
            "\nKeys will refresh automatically every {} seconds.",
            refresh_interval
        );
    }
    eprintln!("\nPress Ctrl+C to stop the agent.");
}

async fn run_agent_with_listener<L>(
    listener: L,
    key_storage: KeyStorage,
    refresh_interval: u64,
    client: &PassClient,
    vault_query: &VaultQuery,
) -> Result<()>
where
    L: ListeningSocket + Send + std::fmt::Debug,
    KeyStorage: ssh_agent_lib::agent::Agent<L>,
{
    if refresh_interval > 0 {
        let mut refresh_interval_timer =
            tokio::time::interval(tokio::time::Duration::from_secs(refresh_interval));
        refresh_interval_timer.tick().await; // Skip the first immediate tick

        tokio::select! {
            result = listen(listener, key_storage.clone()) => {
                result.context("SSH agent error")?;
            }
            _ = async {
                loop {
                    refresh_interval_timer.tick().await;
                    refresh_keys_periodically(client, vault_query, &key_storage, refresh_interval).await;
                }
            } => {}
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
        }
    } else {
        tokio::select! {
            result = listen(listener, key_storage) => {
                result.context("SSH agent error")?;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
        }
    }

    Ok(())
}

pub async fn run(
    socket_path: Option<String>,
    share_id: Option<String>,
    vault_name: Option<String>,
    refresh_interval: u64,
    client: PassClient,
) -> Result<()> {
    let vault_query = VaultQuery::new(share_id, vault_name)?;

    info!("Loading SSH keys from Proton Pass...");

    let identities = load_keys_into_storage(&client, &vault_query)
        .await
        .context("Failed to load SSH keys from vaults")?;

    if identities.is_empty() {
        bail!("No SSH keys found in the specified vault(s)");
    }

    let loaded_count = identities.len();
    info!("Found {} SSH key(s)", loaded_count);

    let key_storage = KeyStorage::default();
    key_storage.replace_all_identities(identities).await;

    eprintln!("Loaded {} SSH key(s) successfully", loaded_count);

    let socket_path = if let Some(path) = socket_path {
        PathBuf::from(path)
    } else {
        get_default_socket_path()?
    };

    if refresh_interval > 0 {
        info!(
            "Automatic key refresh enabled (every {} seconds)",
            refresh_interval
        );
    } else {
        info!("Automatic key refresh disabled");
    }

    #[cfg(unix)]
    {
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).context("Failed to remove existing socket file")?;
        }

        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create socket directory")?;
        }

        // Create Unix socket
        let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

        // Set socket permissions to 0600 (owner read/write only)
        let metadata = std::fs::metadata(&socket_path).context("Failed to get socket metadata")?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(&socket_path, permissions)
            .context("Failed to set socket permissions")?;

        info!("SSH agent listening on: {}", socket_path.display());
        print_agent_startup_message(&socket_path.display().to_string(), refresh_interval);

        let socket_path_clone = socket_path.clone();

        // Run the agent
        run_agent_with_listener(
            listener,
            key_storage,
            refresh_interval,
            &client,
            &vault_query,
        )
        .await?;

        // Cleanup
        if socket_path_clone.exists() {
            std::fs::remove_file(&socket_path_clone).context("Failed to remove socket file")?;
        }
    }

    #[cfg(windows)]
    {
        // On Windows, use a named pipe
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
        let pipe_name = format!(r"\\.\pipe\proton-pass-agent-{}", username);

        info!("SSH agent listening on: {}", pipe_name);
        print_agent_startup_message(&socket_path.display().to_string(), refresh_interval);

        let listener = NamedPipeListener::bind(&pipe_name).context("Failed to bind named pipe")?;

        // Run the agent
        run_agent_with_listener(
            listener,
            key_storage,
            refresh_interval,
            &client,
            &vault_query,
        )
        .await?;
    }

    eprintln!("SSH agent stopped.");

    Ok(())
}
