use crate::commands::Role;
use anyhow::{Context, Result, anyhow};
use pass::PassClient;
use pass_domain::{ShareId, ShareRole};

pub enum ShareVaultQuery {
    ShareId(ShareId),
    VaultName(String),
}

impl ShareVaultQuery {
    pub fn new(share_id: Option<String>, name: Option<String>) -> Result<Self> {
        match (share_id, name) {
            (Some(share_id), None) => Ok(Self::ShareId(ShareId::new(share_id))),
            (None, Some(vault_name)) => Ok(Self::VaultName(vault_name)),

            _ => Err(anyhow!("Please provide either share-id or vault name")),
        }
    }
}

pub async fn run(
    client: PassClient,
    query: ShareVaultQuery,
    email: String,
    role: Role,
) -> Result<()> {
    let share_id = match query {
        ShareVaultQuery::ShareId(id) => id,
        ShareVaultQuery::VaultName(vault) => {
            let vault = client
                .find_vault(&vault)
                .await
                .context("Error finding vault")?;
            vault.share_id
        }
    };
    client
        .share_vault(&share_id, &email, &ShareRole::from(role))
        .await
        .context("Error sharing vault")?;

    Ok(())
}
