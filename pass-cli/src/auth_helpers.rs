use crate::cli_credential_provider::CliCredentialProvider;
use crate::features::CliClientFeatures;
use crate::terminal_event_handler::TerminalEventHandler;
use anyhow::Result;
use pass_auth::{Authenticator, ClientConfig};
use std::sync::Arc;

pub fn create_client_config() -> Result<ClientConfig> {
    Ok(ClientConfig {
        base_dir: crate::utils::get_base_dir()?,
        environment: std::env::var(pass_auth::ENVIRONMENT_ENV_VAR).ok(),
        proxy_config: pass_auth::ProxyConfig::from_env(),
        debug_config: pass_auth::config::DebugConfig::from_env(),
        app_header: None,
        post_login_config: pass_auth::PostLoginConfig::default(),
    })
}

pub fn create_authenticator(client_features: Arc<CliClientFeatures>) -> Result<Authenticator> {
    let config = create_client_config()?;

    Ok(Authenticator::new(
        client_features.key_provider.clone(),
        Arc::new(TerminalEventHandler),
        Arc::new(CliCredentialProvider),
        config,
    ))
}
