use crate::PassClient;
use crate::common::CodeResponse;
use anyhow::{Context, Result};
use muon::DELETE;
use pass_domain::{PersonalAccessTokenId, ShareId};

impl PassClient {
    pub async fn revoke_personal_access_token_access(
        &self,
        personal_access_token_id: &PersonalAccessTokenId,
        share_id: &ShareId,
    ) -> Result<()> {
        self.personal_access_token_operation_guard()?;
        info!(
            "Revoking personal access token {personal_access_token_id} access from share {share_id}"
        );

        let res = self
            .send(DELETE!(
                "/pass/v1/personal-access-token/{}/access/{}",
                personal_access_token_id,
                share_id.value()
            ))
            .await
            .context("Failed to revoke personal access token access")?;

        let response: CodeResponse = assert_response!(res);
        response.success_guard()?;

        info!(
            "Personal access token {} access revoked successfully from share {}",
            personal_access_token_id, share_id
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_tools::*;
    use std::sync::Arc;

    use muon::test::server::{HTTP, Server};

    #[muon::test(scheme(HTTP))]
    async fn test_revoke_access(server: Arc<Server>) {
        const PERSONAL_ACCESS_TOKEN_ID: &str = "test_sa_id";
        const SHARE_ID: &str = "test_share_id";
        const REVOKE_PATH: &str = "/pass/v1/personal-access-token/test_sa_id/access/test_share_id";

        let client = server.pass_client().await;

        let revoke_handled =
            server.handler_with_method(Method::DELETE, REVOKE_PATH, |_| success_code());

        client
            .revoke_personal_access_token_access(
                &PersonalAccessTokenId::new(PERSONAL_ACCESS_TOKEN_ID.to_string()),
                &ShareId::new(SHARE_ID.to_string()),
            )
            .await
            .expect("Should be able to revoke access");

        assert_hit!(revoke_handled);
    }
}
